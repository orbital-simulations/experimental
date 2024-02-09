use crate::{pipeline::CreatePipeline, context::{Context, RenderingContext}};

pub trait RenderBundle {
    fn create_pipeline() -> CreatePipeline<'static>;
    fn draw(&self, context: &Context, render_context: &RenderingContext);
}
