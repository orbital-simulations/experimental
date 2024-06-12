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
    actor_type: SceneNodeType,
    tranform: Transform,
    child_actors: Vec<SceneNode>,
}

impl SceneNode {
    pub fn new() -> Self {
        Self {
            actor_type: SceneNodeType::Invisible,
            tranform: Transform::IDENTITY,
            child_actors: vec![],
        }
    }

    pub fn from_circle(transform: Transform, circle: Circle) -> Self {
        Self {
            actor_type: SceneNodeType::Circle(circle),
            tranform: transform,
            child_actors: vec![],
        }
    }

    pub fn from_circle_children(
        transform: Transform,
        circle: Circle,
        child_actors: Vec<SceneNode>,
    ) -> Self {
        Self {
            actor_type: SceneNodeType::Circle(circle),
            tranform: transform,
            child_actors,
        }
    }

    pub fn from_circle_line(transform: Transform, circle_line: CircleLine) -> Self {
        Self {
            actor_type: SceneNodeType::CircleLine(circle_line),
            tranform: transform,
            child_actors: vec![],
        }
    }

    pub fn from_circle_line_children(
        transform: Transform,
        circle_line: CircleLine,
        child_actors: Vec<SceneNode>,
    ) -> Self {
        Self {
            actor_type: SceneNodeType::CircleLine(circle_line),
            tranform: transform,
            child_actors,
        }
    }

    pub fn from_mesh_bundle(transform: Transform, mesh_bundle: MeshBundle) -> Self {
        Self {
            actor_type: SceneNodeType::MeshBundle(mesh_bundle),
            tranform: transform,
            child_actors: vec![],
        }
    }

    pub fn from_mesh_bundle_children(
        transform: Transform,
        mesh_bundle: MeshBundle,
        child_actors: Vec<SceneNode>,
    ) -> Self {
        Self {
            actor_type: SceneNodeType::MeshBundle(mesh_bundle),
            tranform: transform,
            child_actors,
        }
    }

    pub fn from_rectangle(transform: Transform, rectangle: Rectangle) -> Self {
        Self {
            actor_type: SceneNodeType::Rectangle(rectangle),
            tranform: transform,
            child_actors: vec![],
        }
    }

    pub fn from_rectangle_children(
        transform: Transform,
        rectangle: Rectangle,
        child_actors: Vec<SceneNode>,
    ) -> Self {
        Self {
            actor_type: SceneNodeType::Rectangle(rectangle),
            tranform: transform,
            child_actors,
        }
    }

    pub fn from_rectangle_line(transform: Transform, rectangle_line: RectangleLine) -> Self {
        Self {
            actor_type: SceneNodeType::RectangleLine(rectangle_line),
            tranform: transform,
            child_actors: vec![],
        }
    }

    pub fn from_rectangle_line_children(
        transform: Transform,
        rectangle_line: RectangleLine,
        child_actors: Vec<SceneNode>,
    ) -> Self {
        Self {
            actor_type: SceneNodeType::RectangleLine(rectangle_line),
            tranform: transform,
            child_actors,
        }
    }

    pub fn from_line(transform: Transform, line: Line) -> Self {
        Self {
            actor_type: SceneNodeType::Line(line),
            tranform: transform,
            child_actors: vec![],
        }
    }

    pub fn from_line_children(
        transform: Transform,
        line: Line,
        child_actors: Vec<SceneNode>,
    ) -> Self {
        Self {
            actor_type: SceneNodeType::Line(line),
            tranform: transform,
            child_actors,
        }
    }

    pub fn invisible(transform: Transform, child_actors: Vec<SceneNode>) -> Self {
        Self {
            actor_type: SceneNodeType::Invisible,
            tranform: transform,
            child_actors,
        }
    }

    fn draw_actor(
        world_transform: Transform,
        actor_type: &SceneNodeType,
        line_rendering: &mut LineRenderering,
        rectangle_rndering: &mut RectangleRendering,
        mesh_rendering: &mut MeshRendering,
        circle_rendering: &mut CircleRendering,
    ) {
        match actor_type {
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

    fn draw_child_actor(
        transform: &Transform,
        actor: &SceneNode,
        line_rendering: &mut LineRenderering,
        rectangle_rendering: &mut RectangleRendering,
        mesh_rendering: &mut MeshRendering,
        circle_rendering: &mut CircleRendering,
    ) {
        let world_transform = transform * &actor.tranform;
        Self::draw_actor(
            world_transform,
            &actor.actor_type,
            line_rendering,
            rectangle_rendering,
            mesh_rendering,
            circle_rendering,
        );
        for child_actor in &actor.child_actors {
            SceneNode::draw_child_actor(
                &world_transform,
                child_actor,
                line_rendering,
                rectangle_rendering,
                mesh_rendering,
                circle_rendering,
            );
        }
    }

    pub fn draw_actors(
        actor: &SceneNode,
        line_rendering: &mut LineRenderering,
        rectangle_rendering: &mut RectangleRendering,
        mesh_rendering: &mut MeshRendering,
        circle_rendering: &mut CircleRendering,
    ) {
        let world_transform = actor.tranform;
        Self::draw_actor(
            actor.tranform,
            &actor.actor_type,
            line_rendering,
            rectangle_rendering,
            mesh_rendering,
            circle_rendering,
        );
        for child_actor in &actor.child_actors {
            SceneNode::draw_child_actor(
                &world_transform,
                child_actor,
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
