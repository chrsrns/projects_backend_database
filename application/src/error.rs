#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplicationError {
    Unauthorized,
    Forbidden,
    NotFound(String),
    Conflict(String),
    BadRequest(String),
    Internal(String),
}
