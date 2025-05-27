use thiserror::Error;

#[derive(Debug, Error)]
pub enum CompositorCoreError {
    #[error("Example error: {0}")]
    ExampleError(String),
}
