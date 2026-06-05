use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json};
use std::{env, fs::File, io::read_to_string, path::PathBuf, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum FunctionName {
    Read,
    Unknown,
}

impl From<&str> for FunctionName {
    fn from(s: &str) -> Self {
        match s {
            "Read" => Self::Read,
            _ => Self::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ReadArgs {
    file_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ToolFunction {
    name: FunctionName,
    arguments: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ToolCall {
    id: String,
    r#type: String,
    function: ToolFunction,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: Option<String>,
    // TODO: this is the only thing I am really calling clone on...
    tool_calls: Option<Vec<ToolCall>>,
    tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Choice {
    index: usize,
    message: Message,
    finish_reason: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Response {
    choices: Vec<Choice>,
}

async fn call_api(messages: &[Message]) -> Option<Response> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut messages: Vec<Message> = vec![Message {
        role: "user".to_string(),
        content: Some(args.prompt.to_string()),
        tool_calls: None,
        tool_call_id: None,
    }];

    loop {
        if let Some(response) = call_api(messages.as_slice()).await
            && !response.choices.is_empty()
        {
            eprintln!("Response {:?}", response);
            for choice in response.choices {
                if choice.index == 0 && choice.finish_reason == "stop" {
                    if let Some(content) = choice.message.content {
                        println!("{}", content);
                    }
                    return Ok(());
                }

                // TODO: could I have a message builder here...
                messages.push(Message {
                    role: choice.message.role,
                    content: choice.message.content,
                    tool_calls: choice.message.tool_calls.clone(),
                    tool_call_id: choice.message.tool_call_id,
                });

                if let Some(tool_calls) = &choice.message.tool_calls {
                    for tool_call in tool_calls {
                        match tool_call.function.name {
                            FunctionName::Read => {
                                if let Ok(read_args) =
                                    from_str::<ReadArgs>(&tool_call.function.arguments)
                                    && let Ok(file) = File::open(&read_args.file_path)
                                {
                                    messages.push(Message {
                                        role: "tool".to_string(),
                                        content: Some(read_to_string(file)?),
                                        tool_call_id: Some(tool_call.id.to_string()),
                                        tool_calls: None,
                                    });
                                }
                            }
                            _ => {
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}
