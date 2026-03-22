use async_openai::{Client, config::OpenAIConfig};
use clap::Parser;
use serde_json::{Value, Value::Array, json};
use std::{env, process};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
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
    #[allow(unused_variables)]
    let response: Value = client
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

    // You can use print statements as follows for debugging, they'll be visible when running tests.
    eprintln!("Logs from your program will appear here!");

    if let Value::Array(tool_calls) = &response["choices"][0]["message"]["tool_calls"] {
        if !tool_calls.is_empty() {
            for tool in tool_calls {
                if tool["function"]["name"] == "Read" {
                    if let Some(args) = tool["function"]["arguments"].as_str() {
                        let args: Value = serde_json::from_str(args)?;
                        // TODO: essentially, I am going to use io to read the
                        // file contents from args.file_path and print to std_out
                        // This should prolly all be cleaned up/typed before submission
                        // Struct for args, clippy lints fixed etc, etc
                        todo!()
                    }
                }
            }
        } else {
            if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
                println!("{}", content);
            }
        }
    }

    Ok(())
}
