use std::{env::args, iter, path::Path, sync::Arc};

use color_eyre::eyre::Result;
use eyre::OptionExt;
use glam::Vec2;
use image::{ImageBuffer, Rgba};
use renderer::{
    camera2::PrimaryCamera,
    gpu_context::GpuContext,
    projection2::{CameraProjection, Orthographic},
    renderer_api::Renderer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use wgpu::util::parse_backends_from_comma_list;

const OUTPUT_HEIGH: u32 = 600;
const OUTPUT_WIDTH: u32 = 600;

fn get_program_stem() -> Result<String> {
    let program = args()
        .next()
        .ok_or_eyre("Could not get the first argument")?;
    let path = Path::new(&program);
    let stem = path.file_stem().ok_or_eyre("Could not get the file stem")?;
    let string = stem.to_str().ok_or_eyre("Could not convert to string")?;
    Ok(string.to_owned())
}

pub async fn run<FRender>(render: FRender) -> Result<()>
where
    FRender: Fn(&mut Renderer),
{
    let fmt_layer = tracing_subscriber::fmt::layer().pretty();
    let filter_layer = EnvFilter::from_default_env();
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .init();
    color_eyre::install()?;
    let backends = std::env::var("WGPU_BACKEND")
        .as_deref()
        .map(str::to_lowercase)
        .ok()
        .as_deref()
        .map(parse_backends_from_comma_list)
        .unwrap_or_default();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends,
        ..Default::default()
    });
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .ok_or_eyre("Could not get adapter")?;
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )
        .await?;

    let texture_format = wgpu::TextureFormat::Rgba8UnormSrgb;

    let texture_descriptor = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: OUTPUT_WIDTH,
            height: OUTPUT_HEIGH,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: texture_format,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };

    let texture = device.create_texture(&texture_descriptor);

    // wgpu requires texture -> buffer copies to be aligned using
    // wgpu::COPY_BYTES_PER_ROW_ALIGNMENT. Because of this we'll
    // need to save both the padded_bytes_per_row as well as the
    // unpadded_bytes_per_row.
    let pixel_size: u32 = 4;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bytes_per_row = pixel_size * OUTPUT_WIDTH;
    let padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padding;

    // Create a buffer to copy the texture to so we can get the data.
    let buffer_size = (padded_bytes_per_row * OUTPUT_HEIGH) as wgpu::BufferAddress;
    let output_buffer_descriptor = wgpu::BufferDescriptor {
        size: buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: Some("Output Buffer"),
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_descriptor);

    let gpu_context = Arc::new(GpuContext::new(device, queue));

    let projection = CameraProjection::Orthographic(Orthographic {
        depth: 2.0,
        scale: 1.0,
    });
    let primary_camera = PrimaryCamera {
        projection,
        surface_format: texture_format,
        size: Vec2::new(OUTPUT_WIDTH as f32, OUTPUT_HEIGH as f32),
        depth_buffer: Some(wgpu::ColorTargetState {
            format: wgpu::TextureFormat::Depth32Float,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent::REPLACE,
                alpha: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrites::ALL,
        }),
    };

    let mut renderer = Renderer::new(&gpu_context, primary_camera);

    render(&mut renderer);

    renderer.render(&texture);

    let mut encoder = renderer
        .rendering_context
        .gpu_context
        .device()
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(OUTPUT_HEIGH),
            },
        },
        texture_descriptor.size,
    );
    renderer
        .rendering_context
        .gpu_context
        .queue()
        .submit(iter::once(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);

    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        result.expect("GPU didn't copy data to output buffer");
    });

    renderer
        .rendering_context
        .gpu_context
        .device()
        .poll(wgpu::Maintain::Wait);

    let padded_data = buffer_slice.get_mapped_range();
    let data = padded_data
        .chunks(padded_bytes_per_row as _)
        .flat_map(|chunk| &chunk[..unpadded_bytes_per_row as _])
        .copied()
        .collect::<Vec<_>>();
    let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(OUTPUT_WIDTH, OUTPUT_WIDTH, data)
        .ok_or_eyre("Could not create an image buffer")?;
    let name = get_program_stem()? + ".png";
    println!("Saving rendered image to {}", name);
    buffer.save(name)?;
    Ok(())
}
