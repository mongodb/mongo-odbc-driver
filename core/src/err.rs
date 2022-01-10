use thiserror::Error;

#[derive(Error, Debug)]
pub enum RustCoreError {
    # [error(transparent)]
    RustDriverError(#[from] mongodb::error::Error),  // source and Display delegate to mongodb::Error
}