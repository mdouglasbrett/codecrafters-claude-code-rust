#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]

use async_openai::{Client, config::OpenAIConfig};
use serde_json::json;
use std::{env, process};
use crate::{Message, Response};

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
