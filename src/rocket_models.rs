use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RealtimeMessage {
    pub topic: String,
    pub payload: String,
}

// Implement the `ToString` trait for the struct
impl ToString for RealtimeMessage {
    fn to_string(&self) -> String {
        // Convert the struct to a JSON string using serde_json
        serde_json::to_string(self).unwrap_or_else(|_| "Failed to serialize".to_string())
    }
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct MessageResponse {
    /// This is a message from the server.
    pub message: String,
}
