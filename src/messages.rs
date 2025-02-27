use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandExecutionRequest {
    pub command: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CommandExecutionResponse {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "type")]
pub enum FileOperation {
    Download,
    Upload,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileOperationRequest {
    pub url: String,
    pub path: String,
    pub operation: FileOperation,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileOperationResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "type")]
pub enum ControllerRequestPayload {
    // None,
    CommandExecutionRequest(CommandExecutionRequest),
    FileOperationRequest(FileOperationRequest),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ControllerRequest {
    pub version: u32,
    pub id: u64,
    pub payload: ControllerRequestPayload,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
// #[serde(tag = "type")]
pub enum AgentResponsePayload {
    None,
    CommandExecutionResponse(CommandExecutionResponse),
    FileOperationResponse(FileOperationResponse),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AgentResponse {
    pub id: u64,
    pub ok: bool,
    pub payload: AgentResponsePayload,
}

impl FromStr for ControllerRequest {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

impl ToString for AgentResponse {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
