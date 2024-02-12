use wgpu::{TextureFormat, Texture};

struct RenderTargetDescription<'a> {
    multisampling: u32,
    depth_buffer: Option<Texture>,
    targets: &'a [Texture]
}

