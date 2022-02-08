use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    MongoDriver(#[from] mongodb::error::Error), // Source and Display delegate to mongodb::Error
}
