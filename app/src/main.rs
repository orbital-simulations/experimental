use std::f32::consts::PI;

use game_engine::{
    game_engine_3d_parameters,
    mesh::{generate_mesh_normals, generate_mesh_plane},
    obj_loader::load_model_static,
    GameEngine,
};
use glam::{vec3, Vec3};
use noise::{NoiseFn, SuperSimplex};
use renderer::{
    include_wgsl, mesh_rendering::MeshBundle, resource_store::shader::ShaderSource,
    transform::Transform, Renderer,
};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use winit::{event_loop::EventLoop, window::Window};

pub struct GameState {
    noises: Vec<(u32, f32, f32)>,
    noises_detection: Vec<(u32, f32, f32)>,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
    cube_bundle: MeshBundle,
    reload_cube_bundle: MeshBundle,
    cube_rotation: f32,
    terain_bundle: MeshBundle,
}

const CUBE: &str = include_str!("../assets/cube.obj");
const CUBE_MATERIALS: [(&str, &str); 1] = [("cube.mtl", include_str!("../assets/cube.mtl"))];

fn setup(game_engine: &mut GameEngine) -> GameState {
    let cube_bundle = MeshBundle {
        mesh_id: load_model_static(&mut game_engine.renderer, CUBE, &CUBE_MATERIALS).unwrap(),
        pipeline_id: game_engine
            .renderer
            // TODO: Again think about how far to push the errors
            .create_3d_pipeline(&include_wgsl!("../shaders/cube.wgsl"))
            .unwrap(),
    };

    let reload_cube_bundle = MeshBundle {
        mesh_id: load_model_static(&mut game_engine.renderer, CUBE, &CUBE_MATERIALS).unwrap(),
        pipeline_id: game_engine
            .renderer
            .create_3d_pipeline(&ShaderSource::ShaderFile(
                "app/shaders/cube_reload_test.wgsl".into(),
            ))
            .unwrap(),
    };

    let (mut vertices, indices) = generate_mesh_plane(200, 200, 1.);
    let noise1 = SuperSimplex::new(0);
    let noise2 = SuperSimplex::new(10);
    let noise3 = SuperSimplex::new(100);
    for v in vertices.iter_mut() {
        let mut z = noise1.get([(v.x / 50.) as f64, (v.y / 50.) as f64]) * 25.;
        z += noise2.get([(v.x / 10.) as f64, (v.y / 10.) as f64]) * 3.;
        z += noise3.get([v.x as f64, v.y as f64]) * 0.1;
        v.z = z as f32;
    }

    let normals = generate_mesh_normals(&vertices, &indices);
    let terain_bundle = MeshBundle {
        mesh_id: game_engine
            .renderer
            .rendering_context
            .resource_store
            .build_gpu_mesh(&vertices, &normals, &indices),
        pipeline_id: game_engine
            .renderer
            .create_3d_pipeline(&include_wgsl!("../shaders/terain.wgsl"))
            .unwrap(),
    };

    GameState {
        noises: vec![(0, 50., 25.), (10, 10., 3.), (100, 1., 0.1)],
        vertices,
        indices,
        noises_detection: vec![],
        cube_bundle,
        reload_cube_bundle,
        terain_bundle,
        cube_rotation: 0.0,
    }
}

fn update(state: &mut GameState, game_engine: &mut GameEngine) {
    let egui_context = game_engine.egui();
    egui::SidePanel::right("panel").show(egui_context, |ui| {
        ui.heading("Terrain controls");
        for (index, (seed, scale, strength)) in state.noises.iter_mut().enumerate() {
            ui.label(format!("perlin noice: {}", index));
            ui.add(egui::Slider::new(seed, 1..=1000).text("seed"));
            ui.add(egui::Slider::new(scale, 0.001..=1000.0).text("inverted scale"));
            ui.add(egui::Slider::new(strength, 0.0001..=20.0).text("strength"));
        }
        if ui.button("remove last noise").clicked() {
            state.noises.pop();
        }
        if ui.button("add noise").clicked() {
            state.noises.push((0, 10.0, 0.01));
        }
    });
    if state.noises_detection != state.noises {
        state.noises_detection.clone_from(&state.noises);

        let simplexes: Vec<SuperSimplex> = state
            .noises
            .iter()
            .map(|(seed, _, _)| SuperSimplex::new(*seed))
            .collect();
        for v in state.vertices.iter_mut() {
            let mut z = 0.0;
            for (index, (_, scale, strength)) in state.noises.iter().enumerate() {
                z += simplexes[index].get([(v.x / scale) as f64, (v.y / scale) as f64]) as f32
                    * strength;
            }
            v.z = z;
        }
        let normals = generate_mesh_normals(&state.vertices, &state.indices);
        let gpu_mesh_id = game_engine
            .renderer
            .rendering_context
            .resource_store
            .build_gpu_mesh(&state.vertices, &normals, &state.indices);
        state.terain_bundle.mesh_id = gpu_mesh_id;
    }

    state.cube_rotation += (PI / 180.0) * 2.0;
}

fn render(state: &GameState, renderer: &mut Renderer) {
    renderer.draw_mesh(
        &Transform::from_translation(&vec3(0.0, 0.0, 0.0)),
        &state.terain_bundle,
    );

    let mut cube_transform = Transform::from_rotation_euler(&vec3(0.0, 0.0, state.cube_rotation));
    cube_transform.set_translation(&vec3(-10.0, 100.0, 10.0));

    renderer.draw_mesh(&cube_transform, &state.cube_bundle);
    let mut reload_cube_transform =
        Transform::from_rotation_euler(&vec3(0.0, 0.0, state.cube_rotation));
    reload_cube_transform.set_translation(&vec3(-10.0, 80.0, 10.0));
    renderer.draw_mesh(&reload_cube_transform, &state.reload_cube_bundle);
}

fn main() -> color_eyre::eyre::Result<()> {
    let fmt_layer = fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;
    let event_loop = EventLoop::new()?;
    let window = Window::new(&event_loop)?;
    let (mut game_engine, event_loop) = pollster::block_on(GameEngine::new(
        event_loop,
        &window,
        game_engine_3d_parameters(),
    ))?;
    game_engine.run(event_loop, setup, &update, &render)?;
    Ok(())
}
