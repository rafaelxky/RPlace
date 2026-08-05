use thiserror::Error;

#[derive(Debug,Error)]
#[error("{message}")]
pub struct  NotLoggedInError{
    pub message: String,
}
impl NotLoggedInError {
    pub fn new<T:Into<String>>(message: T) -> Self{
        Self { message: message.into()}
    }
}