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

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
struct ToolFunction {
    name: FunctionName,
    arguments: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ToolCall {
    id: String,
    r#type: String,
    function: ToolFunction,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: Option<String>,
    tool_calls: Vec<ToolCall>,
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

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
    let response: Response = client
        .chat()
        .create_byot(json!({
                    "messages": [
                        {
                            "role": "user",
                            "content": args.prompt
                        }
                    ],
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
        .await?;

    if !response.choices.is_empty()
        && let Some(choice) = response.choices.first()
    {
        let tool_calls = &choice.message.tool_calls;
        for tool_call in tool_calls {
            match tool_call.function.name {
                FunctionName::Read => {
                    eprintln!("{:?}", &tool_call.function.arguments);
                    if let Ok(read_args) = from_str::<ReadArgs>(&tool_call.function.arguments) {
                        if let Ok(file) = File::open(&read_args.file_path) {
                            println!("{}", read_to_string(file)?);
                        }
                    }
                }
                _ => {
                    continue;
                }
            }
        }
    }

    Ok(())
}
