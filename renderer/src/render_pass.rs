use wgpu::TextureFormat;

#[derive(Debug, Clone)]
pub struct RenderTargetDescription {
    pub multisampling: u32,
    pub depth_texture: Option<TextureFormat>,
    pub targets: Vec<TextureFormat>,
}

//// TODO: Maybe this should be an enum to make two case on for normal sampling
//// and one for multi sampling???
//
//pub struct RenderTarget {
//    name: String,
//    multisampling: u32,
//    depth_texture: Option<(Rc<Texture>, Operations<f32>, TextureView)>,
//    // FIXME: This is ugly hack, there should be a way to easily swap the
//    // window surface textures.
//    pub targets: Vec<(Rc<Texture>,Operations<Color>, TextureView)>,
//    multisampling_textures: Vec<(Texture, TextureView)>,
//}
//
//impl RenderTarget {
//    pub fn new(context: &Context, parameters: &RenderTargetDescription) -> Self {
//
//        if parameters.multisampling > 1 {
//
//            let multisampling_textures = parameters.targets.iter().map(|(texture_format, _)|{
//                let texture_size = wgpu::Extent3d {
//                    width: texture.width() * parameters.multisampling,
//                    height: texture.height() * parameters.multisampling,
//                    depth_or_array_layers: 1,
//                };
//                // TODO: The texture should have some lable with it and it
//                // should be used. Sadly the wgpu::Texture doesn't expose
//                // it so we need to make a wrapper or something.
//                let depth_texture_description = wgpu::TextureDescriptor {
//                    label: Some("multisampling texture"),
//                    size: texture_size,
//                    mip_level_count: 1,
//                    sample_count: 1,
//                    dimension: wgpu::TextureDimension::D2,
//                    format: *texture_format,
//                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
//                    view_formats: &[TextureFormat::Depth32Float],
//                };
//                let multisampling_texture = context.device.create_texture(&depth_texture_description);
//                let multisampling_texture_view = multisampling_texture.create_view(&TextureViewDescriptor::default());
//                (multisampling_texture, multisampling_texture_view)
//            }).collect();
//            let depth_texture = parameters.depth_texture.as_ref().map(|(t, o)| {
//                let depth_texture = t.clone();
//                let operations = *o;
//                let texture_view = depth_texture.create_view(&TextureViewDescriptor::default());
//                (depth_texture, operations, texture_view)
//            });
//            let targets = parameters.targets.iter().map(|(t, o)|{
//                let texture = t.clone();
//                let operations = *o;
//                let texture_view = texture.create_view(&TextureViewDescriptor::default());
//                (texture, operations, texture_view)
//            }).collect();
//            Self {
//                name: parameters.name.clone(),
//                multisampling: parameters.multisampling,
//                depth_texture,
//                targets,
//                multisampling_textures,
//            }
//        } else {
//            let depth_texture = parameters.depth_texture.as_ref().map(|(t, o)| {
//                let depth_texture = t.clone();
//                let operations = *o;
//                let texture_view = depth_texture.create_view(&TextureViewDescriptor::default());
//                (depth_texture, operations, texture_view)
//            });
//            let targets = parameters.targets.iter().map(|(t, o)|{
//                let depth_texture = t.clone();
//                let operations = o;
//                let texture_view = depth_texture.create_view(&TextureViewDescriptor::default());
//                (depth_texture, *operations, texture_view)
//            }).collect();
//            Self {
//                multisampling: parameters.multisampling,
//                depth_texture,
//                targets,
//                multisampling_textures: vec![],
//                name: parameters.name.clone(),
//            }
//        }
//    }
//
//    pub fn targets(&self) -> &[(Rc<Texture>,Operations<Color>, TextureView)]{
//        &self.targets
//    }
//
//    pub fn multisampling(&self) -> u32 {
//        self.multisampling
//    }
//
//    pub fn create_render_pass<'a>(&'a self, encoder: &'a mut CommandEncoder) -> RenderPass{
//        if self.multisampling > 1 {
//            let mut color_attachments:  Vec<Option<RenderPassColorAttachment>> = vec![];
//            for (index, (_, operations, texture_view)) in self.targets.iter().enumerate() {
//                color_attachments.push(Some(RenderPassColorAttachment{
//                    view: &self.multisampling_textures[index].1,
//                    resolve_target: Some(texture_view),
//                    ops: *operations,
//                }));
//            }
//
//            let depth_stencil_attachment = self.depth_texture.as_ref().map(|(_, operations, texture_view)| {
//                    RenderPassDepthStencilAttachment {
//                    view: texture_view,
//                    depth_ops: Some(*operations),
//                    stencil_ops: None,
//                    }
//                });
//            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                label: Some("Shapes Renderer Pass"),
//                color_attachments: &color_attachments,
//                depth_stencil_attachment,
//                timestamp_writes: None,
//                occlusion_query_set: None,
//            })
//        } else {
//            let mut color_attachments:  Vec<Option<RenderPassColorAttachment>> = vec![];
//            for (_, operations, texture_view) in self.targets.iter() {
//                color_attachments.push(Some(RenderPassColorAttachment{
//                    view: texture_view,
//                    resolve_target: None,
//                    ops: *operations,
//                }));
//            }
//            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
//                label: Some("Shapes Renderer Pass"),
//                color_attachments: &color_attachments,
//                depth_stencil_attachment: None,
//                timestamp_writes: None,
//                occlusion_query_set: None,
//            })
//        }
//    }
//
//    pub fn resize(&mut self, size: UVec2) {
//        todo!();
//    }
//}
