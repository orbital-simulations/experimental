use crate::{
    circle_rendering::{Circle, CircleLine, CircleRendering},
    line_rendering::{Line, LineRenderering},
    mesh_rendering::{MeshBundle, MeshRendering},
    rectangle_rendering::{Rectangle, RectangleLine, RectangleRendering},
    transform::Transform,
};

#[derive(Debug)]
enum SceneNodeType {
    Circle(Circle),
    CircleLine(CircleLine),
    MeshBundle(MeshBundle),
    Rectangle(Rectangle),
    RectangleLine(RectangleLine),
    Line(Line),
    Invisible,
}

#[derive(Debug)]
pub struct SceneNode {
    node_type: SceneNodeType,
    tranform: Transform,
    child_nodes: Vec<SceneNode>,
}

impl SceneNode {
    pub fn new() -> Self {
        Self {
            node_type: SceneNodeType::Invisible,
            tranform: Transform::IDENTITY,
            child_nodes: vec![],
        }
    }

    pub fn from_circle(transform: Transform, circle: Circle) -> Self {
        Self {
            node_type: SceneNodeType::Circle(circle),
            tranform: transform,
            child_nodes: vec![],
        }
    }

    pub fn from_circle_children(
        transform: Transform,
        circle: Circle,
        child_nodes: Vec<SceneNode>,
    ) -> Self {
        Self {
            node_type: SceneNodeType::Circle(circle),
            tranform: transform,
            child_nodes,
        }
    }

    pub fn from_circle_line(transform: Transform, circle_line: CircleLine) -> Self {
        Self {
            node_type: SceneNodeType::CircleLine(circle_line),
            tranform: transform,
            child_nodes: vec![],
        }
    }

    pub fn from_circle_line_children(
        transform: Transform,
        circle_line: CircleLine,
        child_nodes: Vec<SceneNode>,
    ) -> Self {
        Self {
            node_type: SceneNodeType::CircleLine(circle_line),
            tranform: transform,
            child_nodes,
        }
    }

    pub fn from_mesh_bundle(transform: Transform, mesh_bundle: MeshBundle) -> Self {
        Self {
            node_type: SceneNodeType::MeshBundle(mesh_bundle),
            tranform: transform,
            child_nodes: vec![],
        }
    }

    pub fn from_mesh_bundle_children(
        transform: Transform,
        mesh_bundle: MeshBundle,
        child_nodes: Vec<SceneNode>,
    ) -> Self {
        Self {
            node_type: SceneNodeType::MeshBundle(mesh_bundle),
            tranform: transform,
            child_nodes,
        }
    }

    pub fn from_rectangle(transform: Transform, rectangle: Rectangle) -> Self {
        Self {
            node_type: SceneNodeType::Rectangle(rectangle),
            tranform: transform,
            child_nodes: vec![],
        }
    }

    pub fn from_rectangle_children(
        transform: Transform,
        rectangle: Rectangle,
        child_nodes: Vec<SceneNode>,
    ) -> Self {
        Self {
            node_type: SceneNodeType::Rectangle(rectangle),
            tranform: transform,
            child_nodes,
        }
    }

    pub fn from_rectangle_line(transform: Transform, rectangle_line: RectangleLine) -> Self {
        Self {
            node_type: SceneNodeType::RectangleLine(rectangle_line),
            tranform: transform,
            child_nodes: vec![],
        }
    }

    pub fn from_rectangle_line_children(
        transform: Transform,
        rectangle_line: RectangleLine,
        child_nodes: Vec<SceneNode>,
    ) -> Self {
        Self {
            node_type: SceneNodeType::RectangleLine(rectangle_line),
            tranform: transform,
            child_nodes,
        }
    }

    pub fn from_line(transform: Transform, line: Line) -> Self {
        Self {
            node_type: SceneNodeType::Line(line),
            tranform: transform,
            child_nodes: vec![],
        }
    }

    pub fn from_line_children(
        transform: Transform,
        line: Line,
        child_nodes: Vec<SceneNode>,
    ) -> Self {
        Self {
            node_type: SceneNodeType::Line(line),
            tranform: transform,
            child_nodes,
        }
    }

    pub fn invisible(transform: Transform, child_nodes: Vec<SceneNode>) -> Self {
        Self {
            node_type: SceneNodeType::Invisible,
            tranform: transform,
            child_nodes,
        }
    }

    fn draw_node(
        world_transform: Transform,
        node_type: &SceneNodeType,
        line_rendering: &mut LineRenderering,
        rectangle_rndering: &mut RectangleRendering,
        mesh_rendering: &mut MeshRendering,
        circle_rendering: &mut CircleRendering,
    ) {
        match node_type {
            SceneNodeType::Circle(circle) => {
                circle_rendering.add_circle(&world_transform, circle);
            }
            SceneNodeType::CircleLine(circle_line) => {
                circle_rendering.add_circle_line(&world_transform, circle_line);
            }
            SceneNodeType::MeshBundle(mesh_bundle) => {
                mesh_rendering.add_mesh_bundle(&world_transform, mesh_bundle);
            }
            SceneNodeType::Rectangle(rectangle) => {
                rectangle_rndering.add_rectangle(&world_transform, rectangle);
            }
            SceneNodeType::RectangleLine(rectangle_line) => {
                rectangle_rndering.add_rectangle_line(&world_transform, rectangle_line);
            }
            SceneNodeType::Line(line) => {
                line_rendering.add_line_segment(&world_transform, line);
            }
            SceneNodeType::Invisible => {}
        }
    }

    fn draw_child_node(
        transform: &Transform,
        node: &SceneNode,
        line_rendering: &mut LineRenderering,
        rectangle_rendering: &mut RectangleRendering,
        mesh_rendering: &mut MeshRendering,
        circle_rendering: &mut CircleRendering,
    ) {
        let world_transform = transform * &node.tranform;
        Self::draw_node(
            world_transform,
            &node.node_type,
            line_rendering,
            rectangle_rendering,
            mesh_rendering,
            circle_rendering,
        );
        for child_node in &node.child_nodes {
            SceneNode::draw_child_node(
                &world_transform,
                child_node,
                line_rendering,
                rectangle_rendering,
                mesh_rendering,
                circle_rendering,
            );
        }
    }

    pub fn draw_nodes(
        node: &SceneNode,
        line_rendering: &mut LineRenderering,
        rectangle_rendering: &mut RectangleRendering,
        mesh_rendering: &mut MeshRendering,
        circle_rendering: &mut CircleRendering,
    ) {
        let world_transform = node.tranform;
        Self::draw_node(
            node.tranform,
            &node.node_type,
            line_rendering,
            rectangle_rendering,
            mesh_rendering,
            circle_rendering,
        );
        for child_node in &node.child_nodes {
            SceneNode::draw_child_node(
                &world_transform,
                child_node,
                line_rendering,
                rectangle_rendering,
                mesh_rendering,
                circle_rendering,
            );
        }
    }
}

impl Default for SceneNode {
    fn default() -> Self {
        Self::new()
    }
}
