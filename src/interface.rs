use std::{env::var, process::Command};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RequestError {
    MissingContext(String),
}

pub struct Request {
    command: String,
    error: Option<String>,
    context: Option<String>,
    justification_requested: Option<String>,
}

impl Request {
    pub fn new(
        command: &str,
        error: Option<&str>,
        context: Option<&str>,
        justification_requested: bool,
    ) -> Result<Self, RequestError> {
        //validation
        if !(error.is_some() || context.is_some()) {
            return Err(RequestError::MissingContext(String::from("Your command was successful (no error output), but no context was provided. Please provide context now: ")));
        }
        Ok(Request {
            command: String::from("[command]: ") + command,
            error: error.map(|e| String::from("[error]: ") + e),
            context: context.map(|c| String::from("[context]: ") + c),
            justification_requested: if justification_requested {
                Some(String::from("[justification_requested]"))
            } else {
                None
            },
        })
    }

    pub fn to_payload(&self) -> String {
        let mut payload = self.command.to_string();
        if let Some(error) = self.error.as_ref() {
            payload += " ";
            payload += error.as_str();
        }
        if let Some(context) = self.context.as_ref() {
            payload += " ";
            payload += context.as_str();
        }
        if let Some(jr) = self.justification_requested.as_ref() {
            payload += " ";
            payload += jr.as_str();
        }

        payload
    }

    pub fn add_context(&mut self, context: String) {
        self.context = Some(String::from("[context]: ") + context.as_str());
    }

    pub fn compel_justification(&mut self) {
        self.justification_requested = Some(String::from("[justification_requested]"))
    }
}

#[derive(Serialize, Deserialize)]
struct ClaudeContent {
    r#type: String,
    text: String,
}

#[derive(Serialize, Deserialize)]
struct ClaudeResponse {
    content: Vec<ClaudeContent>,
}

#[derive(Debug)]
pub enum ResponseError {
    BadResponse(String),
}

#[derive(Debug)]
pub struct Response {
    pub command: String,
    pub justification: Option<String>,
}

impl Response {
    pub fn from_string(response_text: String) -> Result<Self, ResponseError> {
        let response = serde_json::from_str::<ClaudeResponse>(&response_text)
            .map_err(|e| ResponseError::BadResponse(e.to_string()))?;

        let content = response
            .content
            .first()
            .ok_or_else(|| ResponseError::BadResponse("Empty response content".to_string()))?;

        let response = if content.text.contains("[justification]") {
            // split on [justification] and process both parts
            let parts: Vec<&str> = content.text.split("[justification]: ").collect();
            let command = parts[0]
                .trim_start_matches("[fixed_command]: ")
                .trim()
                .to_string();
            let justification = Some(parts[1].trim().to_string());

            Response {
                command,
                justification,
            }
        } else {
            // just process the command part
            Response {
                command: content
                    .text
                    .trim_start_matches("[fixed_command]: ")
                    .trim()
                    .to_string(),
                justification: None,
            }
        };

        Ok(response)
    }
}

// shell interface
#[derive(Debug)]
pub enum CommandError {
    NoHistoryFile(String),
    Extract(String),
    Run(String),
    RequestConstruction(RequestError),
}

pub fn get_last_command() -> Result<Request, CommandError> {
    // get shell history file path
    let history_file = var("HISTFILE").map_err(|e| CommandError::NoHistoryFile(e.to_string()))?;

    // get last command from history
    let last_cmd = Command::new("tail")
        .arg("-n")
        .arg("2")
        .arg(&history_file)
        .output()
        .map_err(|e| CommandError::Extract(e.to_string()))?;

    let cmd = String::from_utf8_lossy(&last_cmd.stdout)
        .lines() // split into lines
        .next() // get the first line (second-to-last command, not cllmi or cargo run)
        .ok_or(CommandError::Extract(String::from(
            "No command found in history file",
        )))?
        .split(";")
        .collect::<Vec<&str>>()
        .last()
        .ok_or(CommandError::Extract(String::from(
            "Command needs to be contentful",
        )))?
        .to_string();
    // rerun the command to get its output
    let output = Command::new("sh")
        .arg("-c")
        .arg(&cmd)
        .output()
        .map_err(|e| CommandError::Run(e.to_string()))?;

    let error_output = String::from_utf8_lossy(&output.stderr).to_string();
    let error_option = if error_output.is_empty() {
        None
    } else {
        Some(error_output.as_str())
    };

    Request::new(cmd.as_str(), error_option, None, false).map_err(CommandError::RequestConstruction)
}
