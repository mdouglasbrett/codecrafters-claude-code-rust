#![warn(clippy::style, clippy::complexity, clippy::perf, clippy::correctness)]

use serde::Deserialize;
use serde_json::{Error, from_str};

mod agent;
mod api;
mod types;

pub fn get_tool_call_args<'a, T>(args: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    from_str(args)
}

pub use {
    agent::{AgentState, agent},
    api::call_api,
    get_tool_call_args as get_args,
    types::{FunctionName, Message, ReadArgs, Response, WriteArgs},
};
