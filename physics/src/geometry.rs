//! This module provides basic shapes and methods for testing overlaps between them.
use glam::{DMat2, DVec2};
use tracing::{instrument, trace, warn};

#[derive(Clone, Debug)]
pub struct Contact {
    pub pos: DVec2,
    pub normal: DVec2,
    pub separation: f64,
}

#[derive(Clone, Debug)]
pub enum Shape {
    Circle(Circle),
    Capsule(Capsule),
    HalfPlane(HalfPlane),
}

impl Shape {
    #[instrument(level = "trace")]
    pub fn test_overlap(&self, other: &Shape) -> Vec<Contact> {
        /*
        Implementation choices:
            0. Contact position is in the same coordinates as inputs.
            1. The normal is outward-facing from `self`.
            2. The contact position is on the self's boundary.
            3. Concentric circles are ignored, for now.
         */
        match (self, other) {
            (Shape::Circle(c1), Shape::Circle(c2)) => {
                c1.test_overlap_with_circle(c2).into_iter().collect()
            }
            (Shape::Circle(c1), Shape::Capsule(c2)) => {
                c1.test_overlap_with_capsule(c2).into_iter().collect()
            }
            (Shape::Circle(c1), Shape::HalfPlane(h2)) => {
                c1.test_overlap_with_half_plane(h2).into_iter().collect()
            }
            (Shape::Capsule(c1), Shape::Circle(c2)) => {
                c1.test_overlap_with_circle(c2).into_iter().collect()
            }
            (Shape::Capsule(c1), Shape::Capsule(c2)) => {
                c1.test_overlap_with_capsule(c2).into_iter().collect()
            }
            (Shape::Capsule(c1), Shape::HalfPlane(h2)) => {
                c1.test_overlap_with_half_plane(h2).into_iter().collect()
            }
            (Shape::HalfPlane(h1), Shape::Circle(c2)) => {
                h1.test_overlap_with_circle(c2).into_iter().collect()
            }
            (Shape::HalfPlane(h1), Shape::Capsule(c2)) => {
                h1.test_overlap_with_capsule(c2).into_iter().collect()
            }
            (Shape::HalfPlane(h1), Shape::HalfPlane(h2)) => {
                h1.test_overlap_with_half_plane(h2).into_iter().collect()
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Circle {
    pub pos: DVec2,
    pub radius: f64,
}

#[derive(Clone, Debug)]
pub struct Capsule {
    pub start: DVec2,
    pub end: DVec2,
    pub radius: f64,
}

impl Capsule {
    pub fn new(pos: DVec2, orientation: f64, length: f64, radius: f64) -> Capsule {
        let axis = DMat2::from_angle(orientation) * DVec2::Y;
        Capsule {
            start: pos + length / 2.0 * axis,
            end: pos - length / 2.0 * axis,
            radius,
        }
    }
}

#[derive(Clone, Debug)]
pub struct HalfPlane {
    pub pos: DVec2,
    pub normal_angle: f64,
}

impl Circle {
    fn try_make_contact(&self, normal: DVec2, separation: f64) -> Option<Contact> {
        // No collision
        if separation > 0.0 {
            None
        }
        // Overlap
        else {
            let pos = self.pos + self.radius * normal;
            Some(Contact {
                pos,
                normal,
                separation,
            })
        }
    }

    pub fn test_overlap_with_circle(&self, other: &Circle) -> Option<Contact> {
        let diff = other.pos - self.pos;
        // TODO: decide how to handle concentricity
        // see https://github.com/orbital-simulations/experimental/issues/54
        let normal = diff.try_normalize()?;
        let distance = diff.length();
        let separation = distance - self.radius - other.radius;
        trace!("Overlap result: normal {normal}, separation {separation}");
        self.try_make_contact(normal, separation)
    }

    pub fn test_overlap_with_capsule(&self, _other: &Capsule) -> Option<Contact> {
        unimplemented!("Overlap check of circle with capsule");
    }

    pub fn test_overlap_with_half_plane(&self, other: &HalfPlane) -> Option<Contact> {
        let diff = other.pos - self.pos;
        let normal = -DVec2::from_angle(other.normal_angle);
        let separation = diff.dot(normal) - self.radius;
        trace!("Overlap result: normal {normal}, separation {separation}");
        self.try_make_contact(normal, separation)
    }
}

impl Capsule {
    pub fn test_overlap_with_circle(&self, other: &Circle) -> Option<Contact> {
        other.test_overlap_with_capsule(self).map(|mut c| {
            // c.normal points from `other` to `self`, so we need to flip it.
            c.normal = -c.normal;
            c
        })
    }

    pub fn test_overlap_with_capsule(&self, _other: &Capsule) -> Option<Contact> {
        unimplemented!("Overlap check of capsule with capsule")
    }

    // TODO: we should probably have a ContactManifold that can be supported on multiple points
    // and return Option<ContactManifold>.
    pub fn test_overlap_with_half_plane(&self, other: &HalfPlane) -> Vec<Contact> {
        let start_circle = Circle{pos: self.start, radius: self.radius};
        let start_contact = start_circle.test_overlap_with_half_plane(other);
        let end_circle = Circle{pos: self.end, radius: self.radius};
        let end_contact = end_circle.test_overlap_with_half_plane(other);
        match (start_contact, end_contact) {
            (Some(c1), Some(c2)) => {
                vec![c1, c2]
            }
            // TODO: maybe create multiple contacts in this case also if the line segment
            // overlaps the half-plane
            (Some(c), None) => vec!(c),
            (None, Some(c)) => vec!(c),
            (None, None) => vec![]
        }
    }
}

impl HalfPlane {
    pub fn test_overlap_with_circle(&self, other: &Circle) -> Option<Contact> {
        other.test_overlap_with_half_plane(self).map(|mut c| {
            // c.normal points from `other` to `self`, so we need to flip it.
            c.normal = -c.normal;
            c
        })
    }

    pub fn test_overlap_with_capsule(&self, other: &Capsule) -> Vec<Contact> {
        other.test_overlap_with_half_plane(self).into_iter().map(|mut c| {
            // c.normal points from `other` to `self`, so we need to flip it.
            c.normal = -c.normal;
            c
        }).collect()
    }

    pub fn test_overlap_with_half_plane(&self, _other: &HalfPlane) -> Option<Contact> {
        warn!("Half-plane vs half-plane overlap testing not supported");
        None
    }
}
