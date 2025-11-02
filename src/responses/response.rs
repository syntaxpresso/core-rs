#![allow(dead_code)]

use serde::{Deserialize, Serialize};

/// Universal response wrapper for all API operations.
///
/// This struct provides a consistent format for all responses, whether successful or failed.
/// It follows the same pattern as the Java DataTransferObject but with Rust naming conventions.
///
/// # Examples
///
/// ```
/// use syntaxpresso_core::responses::response::Response;
/// use serde::{Serialize, Deserialize};
/// 
/// #[derive(Serialize, Deserialize)]
/// struct MyData {
///     field: String,
/// }
/// 
/// # fn main() -> Result<(), serde_json::Error> {
/// // Successful response with data
/// let response = Response::success("my-command".to_string(), "/path/to/cwd".to_string(), MyData { field: "value".to_string() });
///
/// // Successful response without data
/// let response: Response<()> = Response::success_empty("my-command".to_string(), "/path/to/cwd".to_string());
///
/// // Error response
/// let response: Response<MyData> = Response::error("my-command".to_string(), "/path/to/cwd".to_string(), "Something went wrong".to_string());
///
/// // Convert to JSON
/// let json = response.to_json()?;
/// # Ok(())
/// # }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Response<T> {
  /// Command that was executed
  pub command: String,

  /// Current working directory for the command
  pub cwd: String,

  /// Boolean flag indicating whether the operation succeeded
  pub succeed: bool,

  /// Generic data payload, present only on successful operations
  #[serde(skip_serializing_if = "Option::is_none")]
  pub data: Option<T>,

  /// Error description, present only on failed operations
  #[serde(skip_serializing_if = "Option::is_none")]
  pub error_reason: Option<String>,
}

impl<T> Response<T>
where
  T: Serialize,
{
  /// Creates a successful response with the provided data payload.
  ///
  /// The resulting Response will have `succeed=true`, the provided data, and `error_reason=None`.
  ///
  /// # Arguments
  /// * `command` - The command name that was executed
  /// * `cwd` - The current working directory for the command
  /// * `data` - The success data to include in the response
  ///
  /// # Returns
  /// A new Response representing a successful operation
  pub fn success(command: String, cwd: String, data: T) -> Self {
    Self { command, cwd, succeed: true, data: Some(data), error_reason: None }
  }

  /// Creates a successful response without any data payload.
  ///
  /// The resulting Response will have `succeed=true`, `data=None`, and `error_reason=None`.
  /// Useful for operations that don't need to return data (e.g., delete operations).
  ///
  /// # Arguments
  /// * `command` - The command name that was executed
  /// * `cwd` - The current working directory for the command
  ///
  /// # Returns
  /// A new Response representing a successful operation without data
  pub fn success_empty(command: String, cwd: String) -> Self {
    Self { command, cwd, succeed: true, data: None, error_reason: None }
  }

  /// Creates a failure response with the provided error message.
  ///
  /// The resulting Response will have `succeed=false`, `data=None`, and the provided error reason.
  /// This method should be used when an operation fails and needs to communicate the failure reason.
  ///
  /// # Arguments
  /// * `command` - The command name that was executed
  /// * `cwd` - The current working directory for the command
  /// * `reason` - A descriptive message explaining what went wrong
  ///
  /// # Returns
  /// A new Response representing a failed operation
  ///
  /// # Panics
  /// Panics if reason is empty after trimming whitespace
  pub fn error(command: String, cwd: String, reason: String) -> Self {
    if reason.trim().is_empty() {
      panic!("Error reason cannot be empty");
    }
    Self { command, cwd, succeed: false, data: None, error_reason: Some(reason) }
  }

  /// Serializes this Response to a compact JSON string.
  ///
  /// Uses serde_json to convert the object to JSON format. The output is compact (single-line)
  /// for easy parsing by IDEs and tools. Fields with None values are excluded from the JSON output
  /// due to the `skip_serializing_if` annotations.
  ///
  /// # Examples
  /// ```json
  /// {"succeed":true,"data":{"field":"value"}}
  /// {"succeed":false,"errorReason":"Something went wrong"}
  /// ```
  ///
  /// # Returns
  /// Result containing the JSON string or serialization error
  pub fn to_json(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string(self)
  }

  /// Serializes this Response to a pretty-formatted JSON string.
  ///
  /// Similar to `to_json()` but with indentation and newlines for better readability.
  ///
  /// # Returns
  /// Result containing the pretty JSON string or serialization error
  pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(self)
  }

  /// Checks if this response represents a successful operation.
  ///
  /// # Returns
  /// `true` if the operation succeeded, `false` otherwise
  pub fn is_success(&self) -> bool {
    self.succeed
  }

  /// Checks if this response represents a failed operation.
  ///
  /// # Returns
  /// `true` if the operation failed, `false` otherwise
  pub fn is_error(&self) -> bool {
    !self.succeed
  }

  /// Gets a reference to the data if the response is successful.
  ///
  /// # Returns
  /// `Some(&T)` if successful and has data, `None` otherwise
  pub fn get_data(&self) -> Option<&T> {
    self.data.as_ref()
  }

  /// Gets a reference to the error reason if the response is a failure.
  ///
  /// # Returns
  /// `Some(&String)` if failed, `None` otherwise
  pub fn get_error(&self) -> Option<&String> {
    self.error_reason.as_ref()
  }
}

impl<T> std::fmt::Display for Response<T>
where
  T: Serialize,
{
  /// Formats the response as a compact JSON string.
  ///
  /// If JSON serialization fails, returns a fallback error response.
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.to_json() {
      Ok(json) => write!(f, "{}", json),
      Err(_) => write!(f, r#"{{"succeed":false,"errorReason":"Serialization failed"}}"#),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde::Serialize;

  #[derive(Serialize, Debug, PartialEq, Clone)]
  struct TestData {
    message: String,
    count: u32,
  }

  #[test]
  fn test_success_with_data() {
    let data = TestData { message: "Hello".to_string(), count: 42 };
    let response =
      Response::success("test-command".to_string(), "/test/path".to_string(), data.clone());

    assert_eq!(response.command, "test-command");
    assert_eq!(response.cwd, "/test/path");
    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.get_data(), Some(&data));
    assert_eq!(response.get_error(), None);
  }

  #[test]
  fn test_success_empty() {
    let response: Response<TestData> =
      Response::success_empty("test-command".to_string(), "/test/path".to_string());

    assert_eq!(response.command, "test-command");
    assert_eq!(response.cwd, "/test/path");
    assert!(response.is_success());
    assert!(!response.is_error());
    assert_eq!(response.get_data(), None);
    assert_eq!(response.get_error(), None);
  }

  #[test]
  fn test_error() {
    let error_msg = "Something went wrong".to_string();
    let response: Response<TestData> =
      Response::error("test-command".to_string(), "/test/path".to_string(), error_msg.clone());

    assert_eq!(response.command, "test-command");
    assert_eq!(response.cwd, "/test/path");
    assert!(!response.is_success());
    assert!(response.is_error());
    assert_eq!(response.get_data(), None);
    assert_eq!(response.get_error(), Some(&error_msg));
  }

  #[test]
  #[should_panic(expected = "Error reason cannot be empty")]
  fn test_error_empty_reason() {
    let _response: Response<TestData> =
      Response::error("test-command".to_string(), "/test/path".to_string(), "   ".to_string());
  }

  #[test]
  fn test_json_serialization() {
    let data = TestData { message: "test".to_string(), count: 1 };
    let response = Response::success("test-command".to_string(), "/test/path".to_string(), data);

    let json = response.to_json().unwrap();
    assert!(json.contains(r#""command":"test-command""#));
    assert!(json.contains(r#""cwd":"/test/path""#));
    assert!(json.contains(r#""succeed":true"#));
    assert!(json.contains(r#""message":"test""#));
    assert!(json.contains(r#""count":1"#));
  }

  #[test]
  fn test_error_json_serialization() {
    let response: Response<TestData> = Response::error(
      "test-command".to_string(),
      "/test/path".to_string(),
      "Test error".to_string(),
    );

    let json = response.to_json().unwrap();
    assert!(json.contains(r#""command":"test-command""#));
    assert!(json.contains(r#""cwd":"/test/path""#));
    assert!(json.contains(r#""succeed":false"#));
    assert!(json.contains(r#""errorReason":"Test error""#));
    assert!(!json.contains("data"));
  }

  #[test]
  fn test_display_trait() {
    let response: Response<TestData> = Response::error(
      "test-command".to_string(),
      "/test/path".to_string(),
      "Test error".to_string(),
    );
    let display_output = format!("{}", response);

    assert!(display_output.contains(r#""command":"test-command""#));
    assert!(display_output.contains(r#""cwd":"/test/path""#));
    assert!(display_output.contains(r#""succeed":false"#));
    assert!(display_output.contains(r#""errorReason":"Test error""#));
  }
}
