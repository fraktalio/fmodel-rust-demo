use std::fmt;

use serde::{Deserialize, Serialize};

/// Error message to be returned to the client
#[derive(Serialize, Deserialize)]
pub struct ErrorMessage {
    pub message: String,
}

/// Implement Display for ErrorMessage
impl fmt::Display for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Implement Debug for ErrorMessage
impl fmt::Debug for ErrorMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ErrorMessage: {}", self.message)
    }
}

// Implementing `From<E>` for your error type allows automatic conversion from any error type `E` that implements `std::error::Error` into your custom error type. This is very useful when your code interacts with multiple libraries or modules that produce different error types.
// It enables the use of the `?` operator seamlessly, converting various error types into your unified error type without boilerplate `map_err()` calls.
// Check `queries.rs` for usage example.
impl<E: std::error::Error> From<E> for ErrorMessage {
    fn from(value: E) -> Self {
        ErrorMessage {
            message: value.to_string(),
        }
    }
}
