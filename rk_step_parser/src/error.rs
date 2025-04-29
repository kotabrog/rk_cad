use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("invalid STEP entity at line {lineno}: {line}")]
    InvalidLine { lineno: usize, line: String },
    #[error(transparent)]
    Int(#[from] std::num::ParseIntError),
    #[error("keyword mismatched (expected {expected}, got {got})")]
    Keyword { expected: &'static str, got: String },
    #[error("attribute parse error: {0}")]
    Attr(String),
    #[error("non-finite REAL value (NaN or ±Infinity) is not allowed in STEP")]
    NonFiniteReal,
}
