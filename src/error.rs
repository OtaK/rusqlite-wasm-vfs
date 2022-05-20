#[derive(Debug, thiserror::Error)]
pub enum WasmVFSError {
    #[error("Cannot instantiate WASM File shim because we're not in a web context with IndexedDB")]
    NoSupport,
    #[error("One of the inner locks is poisoned.")]
    PoisonedLock,
    #[error("{0}")]
    WebError(String),
    #[error("{0}")]
    ErrorString(String),
    #[error(transparent)]
    Other(#[from] eyre::Report),
}

pub type WasmVFSResult<T> = Result<T, WasmVFSError>;
