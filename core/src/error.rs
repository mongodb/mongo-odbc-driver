#[derive(Clone, Debug, Error)]
#[error(display = "{}", kind)]
#[non_exhaustive]
pub struct Error {
    /// The type of error that occurred.
    pub kind: Arc<ErrorKind>,
    labels: Vec<String>,
}
