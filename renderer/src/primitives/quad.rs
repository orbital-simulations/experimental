use glam::Vec2;

pub const QUAD_2D_VERICES: [Vec2; 4] = [
    Vec2 { x: -1.0, y: -1.0 },
    Vec2 { x: 1.0, y: -1.0 },
    Vec2 { x: -1.0, y: 1.0 },
    Vec2 { x: 1.0, y: 1.0 },
];

pub const QUAD_2D_INDICES: &[u16] = &[0, 1, 3, 3, 2, 0];
