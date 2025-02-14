#[derive(serde::Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
    pub data: serde_json::Value,
}

impl ApiResponse {
    pub fn ok(message: &str, data: serde_json::Value) -> Self {
        Self {
            status: "success".to_string(),
            message: message.to_string(),
            data,
        }
    }
    pub fn internal_error(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            message: message.to_string(),
            data: serde_json::Value::Null,
        }
    }
    pub fn bad_request(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            message: message.to_string(),
            data: serde_json::Value::Null,
        }
    }
}
