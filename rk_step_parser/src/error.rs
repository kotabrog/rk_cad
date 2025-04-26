use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid STEP entity at line {lineno}: {line}")]
    InvalidLine { lineno: usize, line: String },
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
}
