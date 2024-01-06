//! This module provides basic shapes and methods for testing overlaps between them.
use glam::DVec2;
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
            (Shape::Circle(c1), Shape::HalfPlane(h2)) => {
                c1.test_overlap_with_half_plane(h2).into_iter().collect()
            }
            (Shape::HalfPlane(h1), Shape::Circle(c1)) => {
                h1.test_overlap_with_circle(c1).into_iter().collect()
            }
            (Shape::HalfPlane(_h1), Shape::HalfPlane(_h2)) => {
                warn!("Half-plane vs half-plane overlap testing not supported");
                vec![]
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
        let normal = diff.try_normalize()?;
        let distance = diff.length();
        let separation = distance - self.radius - other.radius;
        trace!("Overlap result: normal {normal}, separation {separation}");
        self.try_make_contact(normal, separation)
    }

    pub fn test_overlap_with_half_plane(&self, other: &HalfPlane) -> Option<Contact> {
        let diff = other.pos - self.pos;
        let normal = -DVec2::from_angle(other.normal_angle);
        let separation = diff.dot(normal) - self.radius;
        trace!("Overlap result: normal {normal}, separation {separation}");
        self.try_make_contact(normal, separation)
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
}
