use game_engine::{
    game_engine_3d_parameters,
    mesh::{generate_mesh_normals, generate_mesh_plane},
    obj_loader::load_model_static,
    GameEngine,
};
use glam::vec3;
use noise::{NoiseFn, OpenSimplex};
use renderer::{custom_mesh_renderer::CustomMashRenderer, mesh::GpuMesh, Renderer};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::include_wgsl;
use winit::{event_loop::EventLoop, window::Window};

pub struct GameState();

const CUBE: &str = include_str!("../assets/cube.obj");
const CUBE_MATERIALS: [(&str, &str); 1] = [("cube.mtl", include_str!("../assets/cube.mtl"))];

fn setup(game_engine: &mut GameEngine) -> GameState {
    let shader = game_engine
        .renderer
        .context
        .device
        .create_shader_module(include_wgsl!("../shaders/cube.wgsl"));
    let gpu_mesh = load_model_static(&game_engine.renderer.context, CUBE, &CUBE_MATERIALS).unwrap();
    let custom_renderer = CustomMashRenderer::new(
        &game_engine.renderer.context,
        &game_engine
            .renderer
            .renderer_context
            .common_bind_group_layout,
        gpu_mesh,
        shader,
    );
    game_engine
        .renderer
        .add_custom_mesh_renderer(custom_renderer);

    //    let mut vertices = generate_mesh_plane(200, 200, 1.);
    //
    //    let shader = game_engine
    //        .renderer
    //        .context
    //        .device
    //        .create_shader_module(include_wgsl!("../shaders/terain.wgsl"));
    //    let noise1 = OpenSimplex::new(0);
    //    let noise2 = OpenSimplex::new(10);
    //    let noise3 = OpenSimplex::new(100);
    //    for v in vertices.iter_mut() {
    //        let mut z = noise1.get([(v.x / 50.) as f64, (v.y / 50.) as f64]) * 50.;
    //        z += noise2.get([(v.x / 10.) as f64, (v.y / 10.) as f64]) * 10.;
    //        z += noise3.get([v.x as f64, v.y as f64]);
    //        v.z = z as f32;
    //    }
    //    let normals = generate_mesh_normals(&vertices);
    //
    //    let gpu_mesh = GpuMesh::new(&game_engine.renderer.context, &vertices, &normals);
    //    let custom_renderer = CustomMashRenderer::new(
    //        &game_engine.renderer.context,
    //        &game_engine
    //            .renderer
    //            .renderer_context
    //            .common_bind_group_layout,
    //        gpu_mesh,
    //        shader,
    //    );
    //    game_engine
    //        .renderer
    //        .add_custom_mesh_renderer(custom_renderer);

    //let vertices = [
    //    vec3(-10., -10., -10.),
    //    vec3(10., -10., -10.),
    //    vec3(10., 10., -10.),
    //    vec3(-10., 10., -10.),
    //    vec3(-10., -10., 10.),
    //    vec3(10., -10., 10.),
    //    vec3(10., 10., 10.),
    //    vec3(-10., 10., 10.),
    //];

    GameState()
}

fn update(_tate: &mut GameState, _game_engine: &mut GameEngine) {}

fn render(_state: &GameState, _renderer: &mut Renderer) {}

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
