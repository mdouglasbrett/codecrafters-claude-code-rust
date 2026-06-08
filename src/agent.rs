#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]
use crate::{BashArgs, FunctionName, Message, ReadArgs, WriteArgs, call_api, get_args};
use std::{
    error::Error,
    fs::File,
    io::{Write, read_to_string},
    process::Command,
};

pub enum AgentState {
    Working,
    Finished,
}
pub async fn agent(messages: &mut Vec<Message>) -> Result<AgentState, Box<dyn Error>> {
    if let Some(response) = call_api(messages.as_slice()).await
        && !response.choices.is_empty()
    {
        for choice in response.choices {
            if choice.index == 0 && choice.finish_reason == "stop" {
                if let Some(content) = choice.message.content {
                    println!("{}", content);
                }
                return Ok(AgentState::Finished);
            }

            // TODO: I think I can get rid of this clone if I handle the tool calls first
            // VecDeque and push_front for this item then append to messages?
            let resp_message = Message::builder()
                .role(choice.message.role)
                .content(choice.message.content)
                .tool_calls(choice.message.tool_calls.clone())
                .tool_call_id(choice.message.tool_call_id)
                .build();
            messages.push(resp_message);

            if let Some(tool_calls) = &choice.message.tool_calls {
                for tool_call in tool_calls {
                    match tool_call.function.name {
                        FunctionName::Read => {
                            if let Ok(read_args) =
                                get_args::<ReadArgs>(&tool_call.function.arguments)
                                && let Ok(file) = File::open(&read_args.file_path)
                            {
                                let tool_message = Message::builder()
                                    .role("tool".to_string())
                                    .content(Some(read_to_string(file)?))
                                    .tool_call_id(Some(tool_call.id.to_string()))
                                    .build();
                                messages.push(tool_message);
                            }
                        }
                        FunctionName::Write => {
                            if let Ok(write_args) =
                                get_args::<WriteArgs>(&tool_call.function.arguments)
                            {
                                let mut file = File::create(write_args.file_path)?;
                                file.write_all(write_args.content.as_bytes())?;
                                let tool_message = Message::builder()
                                    .role("tool".to_string())
                                    .content(Some("Created the file".to_string()))
                                    .tool_call_id(Some(tool_call.id.to_string()))
                                    .build();
                                messages.push(tool_message);
                            }
                        }
                        FunctionName::Bash => {
                            if let Ok(bash_args) =
                                get_args::<BashArgs>(&tool_call.function.arguments)
                            {
                                let args = bash_args.command.split(" ").collect::<Vec<&str>>();
                                if let Some((first, rest)) = args.split_first() {
                                    let _command = Command::new(first).args(rest);
                                    todo!();
                                }
                            }
                        }
                        FunctionName::Unknown => {
                            continue;
                        }
                    }
                }
            }
        }
    }
    Ok(AgentState::Working)
}
