use std::rc::Rc;

use game_engine::{
    game_engine_3d_parameters,
    mesh::{generate_mesh_normals, generate_mesh_plane},
    obj_loader::load_model_static,
    GameEngine, Scene,
};
use glam::Vec3;
use noise::{NoiseFn, SuperSimplex};
use renderer::{custom_mesh_renderer::CustomMeshRenderer, mesh::GpuMesh, CustomRenderer};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::{include_wgsl, ShaderModule};
use winit::{event_loop::EventLoop, window::Window};

pub struct GameState {
    noises: Vec<(u32, f32, f32)>,
    noises_detection: Vec<(u32, f32, f32)>,
    terrain_shader: Rc<ShaderModule>,
    vertices: Vec<Vec3>,
    indices: Vec<u32>,
}

const CUBE: &str = include_str!("../assets/cube.obj");
const CUBE_MATERIALS: [(&str, &str); 1] = [("cube.mtl", include_str!("../assets/cube.mtl"))];

struct CubeRenderer;
impl CustomRenderer for CubeRenderer {}
struct TerrainRenderer;
impl CustomRenderer for TerrainRenderer {}

fn setup(game_engine: &mut GameEngine) -> GameState {
    let shader = game_engine
        .context
        .device
        .create_shader_module(include_wgsl!("../shaders/cube.wgsl"));
    let gpu_mesh = load_model_static(&game_engine.context, CUBE, &CUBE_MATERIALS).unwrap();
    let custom_renderer = CustomMeshRenderer::new(gpu_mesh, Rc::new(shader));
    game_engine
        .renderer
        .add_custom_mesh_renderer(&CubeRenderer, custom_renderer);

    let shader = Rc::new(
        game_engine
            .context
            .device
            .create_shader_module(include_wgsl!("../shaders/terain.wgsl")),
    );

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

    let gpu_mesh = GpuMesh::new(&game_engine.context, &vertices, &normals, &indices);
    let custom_renderer = CustomMeshRenderer::new(gpu_mesh, shader.clone());
    game_engine
        .renderer
        .add_custom_mesh_renderer(&TerrainRenderer, custom_renderer);
    GameState {
        noises: vec![(0, 50., 25.), (10, 10., 3.), (100, 1., 0.1)],
        terrain_shader: shader,
        vertices,
        indices,
        noises_detection: vec![],
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

        let gpu_mesh = GpuMesh::new(
            &game_engine.context,
            &state.vertices,
            &normals,
            &state.indices,
        );
        let custom_renderer = CustomMeshRenderer::new(gpu_mesh, state.terrain_shader.clone());
        game_engine
            .renderer
            .add_custom_mesh_renderer(&TerrainRenderer, custom_renderer);
    }
}

fn render(_state: &GameState, _scene: &mut Scene) {}

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
