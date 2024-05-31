use super::{PipelineId, ShaderId};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RebuildCommand {
    Shader(ShaderId),
    Pipeline(PipelineId),
}
