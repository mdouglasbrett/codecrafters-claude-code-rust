#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]
use crate::{FunctionName, Message, ReadArgs, WriteArgs, call_api, get_args};
use std::{error::Error, fs::File, io::read_to_string};

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
                            if let Ok(_write_args) =
                                get_args::<WriteArgs>(&tool_call.function.arguments)
                            {
                                todo!();
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
