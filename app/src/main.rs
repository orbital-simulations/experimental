use game_engine::{
    game_engine_3d_parameters,
    mesh::{generate_mesh_normals, generate_mesh_plane},
    GameEngine,
};
use glam::vec3;
use noise::{NoiseFn, OpenSimplex};
use renderer::{custom_mesh_renderer::CustomMashRenderer, mesh::GpuMesh, Renderer};
use tracing_subscriber::{filter::EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use wgpu::include_wgsl;
use winit::{event_loop::EventLoop, window::Window};

pub struct GameState();

fn setup(game_engine: &mut GameEngine) -> GameState {
    let mut vertices = generate_mesh_plane(200, 200, 1.);

    let shader = game_engine
        .renderer
        .context
        .device
        .create_shader_module(include_wgsl!("../shaders/terain.wgsl"));
    let noise1 = OpenSimplex::new(0);
    let noise2 = OpenSimplex::new(10);
    let noise3 = OpenSimplex::new(100);
    for v in vertices.iter_mut() {
        let mut z = noise1.get([(v.x / 50.) as f64, (v.y / 50.) as f64]) * 50.;
        z += noise2.get([(v.x / 10.) as f64, (v.y / 10.) as f64]) * 10.;
        z += noise3.get([v.x as f64, v.y as f64]);
        v.z = z as f32;
    }
    let normals = generate_mesh_normals(&vertices);

    let gpu_mesh = GpuMesh::new(&game_engine.renderer.context, &vertices, &normals);
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

    let vertices = [
        // Front
        vec3(-10., -10., -10.), // 0
        vec3(10., -10., -10.), // 1
        vec3(10., 10., -10.), // 2

        vec3(10., 10., -10.), // 2
        vec3(-10., 10., -10.), // 3
        vec3(-10., -10., -10.), // 0

        // Back
        vec3(-10., -10., 10.), // 4
        vec3(10., -10., 10.), // 5
        vec3(10., 10., 10.), // 6

        vec3(10., 10., 10.), // 6
        vec3(-10., 10., 10.), // 7
        vec3(-10., -10., 10.), // 4

        // Right
        vec3(10., -10., -10.), // 1
        vec3(10., -10., 10.), // 5
        vec3(10., 10., 10.), // 6

        vec3(10., 10., 10.), // 6
        vec3(10., 10., -10.), // 2
        vec3(10., -10., -10.), // 1

        // Left
        vec3(-10., -10., 10.), // 4
        vec3(-10., -10., -10.), // 0
        vec3(-10., 10., -10.), // 3

        vec3(-10., 10., -10.), // 3
        vec3(-10., 10., 10.), // 7
        vec3(-10., -10., 10.), // 4

        // Top
        vec3(-10., 10., -10.), // 3
        vec3(10., 10., 10.), // 6
        vec3(10., 10., -10.), // 2

        vec3(-10., 10., 10.), // 7
        vec3(10., 10., 10.), // 6
        vec3(-10., 10., -10.), // 3

        // Bottom
        vec3(-10., -10., 10.), // 4
        vec3(10., -10., 10.), // 5
        vec3(10., -10., -10.), // 1

        vec3(10., -10., -10.), // 1
        vec3(-10., -10., -10.), // 0
        vec3(-10., -10., 10.), // 4
    ];

    let normals = [
        // Front
        vec3(0., 0., -1.),
        vec3(0., 0., -1.),
        vec3(0., 0., -1.),
        vec3(0., 0., -1.),
        vec3(0., 0., -1.),
        vec3(0., 0., -1.),

        // Back
        vec3(0., 0., 1.),
        vec3(0., 0., 1.),
        vec3(0., 0., 1.),
        vec3(0., 0., 1.),
        vec3(0., 0., 1.),
        vec3(0., 0., 1.),

        // Right
        vec3(1., 0., 0.),
        vec3(1., 0., 0.),
        vec3(1., 0., 0.),
        vec3(1., 0., 0.),
        vec3(1., 0., 0.),
        vec3(1., 0., 0.),

        // Left
        vec3(-1., 0., 0.),
        vec3(-1., 0., 0.),
        vec3(-1., 0., 0.),
        vec3(-1., 0., 0.),
        vec3(-1., 0., 0.),
        vec3(-1., 0., 0.),

        // Top
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),
        vec3(0., 1., 0.),

        // Bottom
        vec3(0., -1., 0.),
        vec3(0., -1., 0.),
        vec3(0., -1., 0.),
        vec3(0., -1., 0.),
        vec3(0., -1., 0.),
        vec3(0., -1., 0.),
    ];

    let cube_shader = game_engine
        .renderer
        .context
        .device
        .create_shader_module(include_wgsl!("../shaders/cube.wgsl"));
    let cube_mesh = GpuMesh::new(&game_engine.renderer.context, &vertices, &normals);
    let cube_renderer = CustomMashRenderer::new(
        &game_engine.renderer.context,
        &game_engine
            .renderer
            .renderer_context
            .common_bind_group_layout,
        cube_mesh,
        cube_shader,
    );
    game_engine.renderer.add_custom_mesh_renderer(cube_renderer);
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
