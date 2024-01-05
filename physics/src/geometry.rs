//! This module provides basic shapes and methods for testing overlaps between them.
use glam::DVec2;

pub struct Contact {
    pub pos: DVec2,
    pub normal: DVec2,
    pub separation: f64,
}

pub struct Circle {
    pub pos: DVec2,
    pub radius: f64,
}

impl Circle {
    pub fn test_overlap_with_circle(&self, other: &Circle) -> Option<Contact> {
        /*
        Implementation choices:
            0. Contact position is in the same coordinates as inputs.
            1. The normal is outward-facing from `self`.
            2. The contact position is on the self's boundary.
            3. Concentric circles are ignored, for now.
         */
        let diff = other.pos - self.pos;
        // TODO: decide how to handle concentricity
        let normal = diff.try_normalize()?;
        let distance = diff.length();

        let separation = distance - self.radius - other.radius;
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
}
