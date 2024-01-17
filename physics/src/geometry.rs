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
    /// Reports contacts between shapes.
    /// * All coordinates are global
    /// * The normal points from `self` towards `other`.
    /// * The contact position is in the middle of the overlap
    #[instrument(level = "trace")]
    pub fn test_overlap(&self, other: &Shape) -> Vec<Contact> {
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
            let pos = self.pos + (self.radius + separation / 2.0) * normal;
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

    pub fn test_overlap_with_capsule(&self, other: &Capsule) -> Option<Contact> {
        // Find the closest point to self.pos on the capsule's segment and
        // treat this as a collision with a virtual circle at that point.
        let line_segment = other.end - other.start;
        let line_length = line_segment.length();
        let line_normal = line_segment / line_length;
        let fraction = (self.pos - other.start)
            .dot(line_normal)
            .clamp(0.0, line_length);
        let projected_self = other.start + fraction * line_normal;
        self.test_overlap_with_circle(&Circle {
            pos: projected_self,
            radius: other.radius,
        })
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
        // TODO: produce two collision points when the capsule are close to parallel
        // should be similar to overlap testing with half-plane plus some clipping

        None
    }

    // `alpha` is between 0 and 1 and specifies a point between `self.start` and `self.end`
    fn make_contact(&self, alpha: f64, normal: DVec2, separation: f64) -> Contact {
        let pos = self.start.lerp(self.end, alpha);
        Contact {
            pos: pos + (self.radius + separation / 2.0) * normal,
            normal,
            separation,
        }
    }

    // TODO: we should probably have a ContactManifold that can be supported on multiple points
    // and return Option<ContactManifold>.
    pub fn test_overlap_with_half_plane(&self, other: &HalfPlane) -> Vec<Contact> {
        let normal = -DVec2::from_angle(other.normal_angle);
        let start_diff = other.pos - self.start;
        let start_separation = start_diff.dot(normal) - self.radius;
        let end_diff = other.pos - self.end;
        let end_separation = end_diff.dot(normal) - self.radius;
        let make_contact = |alpha, separation| self.make_contact(alpha, normal, separation);
        match (start_separation <= 0.0, end_separation <= 0.0) {
            (true, true) => {
                vec![
                    make_contact(0.0, start_separation),
                    make_contact(1.0, end_separation),
                ]
            }
            (true, false) => {
                let alpha = start_separation / (start_separation - end_separation);
                vec![
                    make_contact(0.0, start_separation),
                    make_contact(alpha, 0.0),
                ]
            }
            (false, true) => {
                let alpha = end_separation / (end_separation - start_separation);
                vec![make_contact(alpha, 0.0), make_contact(1.0, end_separation)]
            }
            (false, false) => vec![],
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
        other
            .test_overlap_with_half_plane(self)
            .into_iter()
            .map(|mut c| {
                // c.normal points from `other` to `self`, so we need to flip it.
                c.normal = -c.normal;
                c
            })
            .collect()
    }

    pub fn test_overlap_with_half_plane(&self, _other: &HalfPlane) -> Option<Contact> {
        // TODO: distinguish static from dynamic shapes
        // infinite shapes should be static and there's no point in testing static vs static
        None
    }
}
