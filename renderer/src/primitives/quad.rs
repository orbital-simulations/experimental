use glam::{vec2, Vec2};

pub const QUAD_2D_VERICES: [Vec2; 4] = [
    vec2 ( -1.0, -1.0 ),
    vec2 ( 1.0, -1.0 ),
    vec2 ( -1.0, 1.0 ),
    vec2 ( 1.0, 1.0 ),
];

pub const QUAD_2D_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];
