use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestGeneratorError {
    #[error("Init err: {0}")]
    Init(String),
}

type Result<T> = std::result::Result<T, TestGeneratorError>;

fn main() -> Result<()> {
    Ok(())
}
