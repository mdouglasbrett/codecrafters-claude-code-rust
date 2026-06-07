#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]
use clap::Parser;
use codecrafters_claude_code::{AgentState, Message, agent};

#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    #[arg(short = 'p', long)]
    prompt: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let seed = Message::builder()
        .content(Some(args.prompt.to_string()))
        .build();
    let mut messages: Vec<Message> = vec![seed];

    loop {
        if let AgentState::Finished = agent(&mut messages).await? {
            break;
        }
    }
    Ok(())
}
