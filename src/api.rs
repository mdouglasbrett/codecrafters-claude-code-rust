#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]

use async_openai::{Client, config::OpenAIConfig};
use serde_json::json;
use serde::{Deserialize, Serialize};
use std::{env, process, path::PathBuf};


#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FunctionName {
    Read,
    Write,
    Unknown,
}

impl From<&str> for FunctionName {
    fn from(s: &str) -> Self {
        match s {
            "Read" => Self::Read,
            "Write" => Self::Write,
            _ => Self::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReadArgs {
    pub file_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WriteArgs {
    pub file_path: PathBuf,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolFunction {
    pub name: FunctionName,
    pub arguments: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: ToolFunction,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Message {
    pub role: String,
    pub content: Option<String>,
    // TODO: this is the only thing I am really calling clone on...
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn builder() -> MessageBuilder {
        MessageBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct MessageBuilder {
    role: Option<String>,
    content: Option<String>,
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
}

impl MessageBuilder {
    fn new() -> Self {
        Self::default()
    }
    pub fn role(mut self, role: String) -> Self {
        self.role = Some(role);
        self
    }
    pub fn content(mut self, content: Option<String>) -> Self {
        self.content = content;
        self
    }
    pub fn tool_calls(mut self, tool_calls: Option<Vec<ToolCall>>) -> Self {
        self.tool_calls = tool_calls;
        self
    }
    pub fn tool_call_id(mut self, tool_call_id: Option<String>) -> Self {
        self.tool_call_id = tool_call_id;
        self
    }
    pub fn build(self) -> Message {
        Message {
            role: self.role.unwrap_or("user".to_string()),
            content: self.content,
            tool_calls: self.tool_calls,
            tool_call_id: self.tool_call_id,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub index: usize,
    pub message: Message,
    pub finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub choices: Vec<Choice>,
}

pub async fn call_api(messages: &[Message]) -> Option<Response> {
    let base_url = env::var("OPENROUTER_BASE_URL")
        .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

    let api_key = env::var("OPENROUTER_API_KEY").unwrap_or_else(|_| {
        eprintln!("OPENROUTER_API_KEY is not set");
        process::exit(1);
    });
    let config = OpenAIConfig::new()
        .with_api_base(base_url)
        .with_api_key(api_key);
    let client = Client::with_config(config);
    let response = client
        .chat()
        .create_byot(json!({
            "messages": messages,
            "model": "anthropic/claude-haiku-4.5",
            "tools": [
            {
                "type": "function",
                "function": {
                    "name": "Read",
                    "description": "Read and return the contents of a file",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "The path to the file to read"
                            }
                        },
                        "required": ["file_path"]
                    }
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "Write",
                    "description": "Write content to a file",
                    "parameters": {
                        "type": "object",
                        "required": ["file_path", "content"],
                        "properties": {
                            "file_path": {
                                "type": "string",
                                "description": "The path of the file to write to"
                            },
                            "content": {
                                "type": "string",
                                "description": "The content to write to the file"
                            }
                        }
                    }
                }
            }
            ]
        }))
        .await;

    if response.is_err() {
        return None;
    }

    // TODO: do I want to panic?
    Some(response.expect("Something went wrong!"))
}
