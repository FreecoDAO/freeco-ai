//! Kernel-specific error types.

use freeco_types::error::FreecoError;
use thiserror::Error;

/// Kernel error type wrapping FreecoError with kernel-specific context.
#[derive(Error, Debug)]
pub enum KernelError {
    /// A wrapped FreecoError.
    #[error(transparent)]
    Freeco(#[from] FreecoError),

    /// The kernel failed to boot.
    #[error("Boot failed: {0}")]
    BootFailed(String),
}

/// Alias for kernel results.
pub type KernelResult<T> = Result<T, KernelError>;
