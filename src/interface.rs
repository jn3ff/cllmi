use std::{
    env::var,
    error::Error,
    fs::OpenOptions,
    io::Write,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum RequestError {
    MissingContext(String),
}

pub struct Request {
    command: String,
    output: Option<String>,
    context: Option<String>,
    justification_requested: Option<String>,
}

impl Request {
    pub fn new(
        command: &str,
        output: Option<&str>,
        context: Option<&str>,
        justification_requested: bool,
    ) -> Self {
        Request {
            command: format!("<command>{}</command>", command),
            output: output.map(|e| format!("<output>{}</output>", e)),
            context: context.map(|c| format!("<context>{}</context>", c)),
            justification_requested: if justification_requested {
                Some(String::from("<justification_requested/>"))
            } else {
                None
            },
        }
    }

    pub fn to_payload(&self) -> Result<String, RequestError> {
        //validation
        if !(self.output.is_some() || self.context.is_some()) {
            return Err(RequestError::MissingContext(String::from("Your command was successful (no output), but no context was provided. Please provide context now: ")));
        }
        let mut payload = self.command.to_string();
        if let Some(output) = self.output.as_ref() {
            payload += " ";
            payload += output.as_str();
        }
        if let Some(context) = self.context.as_ref() {
            payload += " ";
            payload += context.as_str();
        }
        if let Some(jr) = self.justification_requested.as_ref() {
            payload += " ";
            payload += jr.as_str();
        }

        Ok(payload)
    }

    pub fn add_context(&mut self, context: String) {
        self.context = Some(format!("<context>{}</context>", context));
    }

    pub fn compel_justification(&mut self) {
        self.justification_requested = Some(String::from("<justification_requested/>"))
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
    Api(String),
    Parse(String),
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    message: String,
}

#[derive(Deserialize)]
struct ApiError {
    error: ApiErrorDetail,
}

#[derive(Debug)]
pub struct Response {
    pub command: String,
    pub justification: Option<String>,
}

impl Response {
    pub fn from_api_error(response_text: &str) -> ResponseError {
        match serde_json::from_str::<ApiError>(response_text) {
            Ok(api_err) => ResponseError::Api(api_err.error.message),
            Err(_) => ResponseError::Api(response_text.to_string()),
        }
    }

    pub fn from_string(response_text: String) -> Result<Self, ResponseError> {
        let response = serde_json::from_str::<ClaudeResponse>(&response_text)
            .map_err(|e| ResponseError::Parse(format!("{}: {}", e, response_text)))?;

        let content = response
            .content
            .first()
            .ok_or_else(|| ResponseError::Parse("Empty response content".to_string()))?;

        let command = Self::extract_tag_content(&content.text, "fixed_command")
            .ok_or_else(|| ResponseError::Parse("No fixed_command tag found".to_string()))?;

        let justification = Self::extract_tag_content(&content.text, "justification");

        Ok(Response {
            command,
            justification,
        })
    }

    fn extract_tag_content(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        let start = text.find(&open_tag)?;
        let end = text.find(&close_tag)?;

        let content_start = start + open_tag.len();
        if content_start < end {
            Some(text[content_start..end].trim().to_string())
        } else {
            None
        }
    }
}

// shell interface
#[derive(Debug)]
pub enum CommandError {
    NoHistoryFile(String),
    Extract(String),
    Run(String),
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

    Ok(Request::new(cmd.as_str(), error_option, None, false))
}

pub fn run_command_with_history(command: &String) -> Result<Output, Box<dyn Error>> {
    let output = Command::new("sh").arg("-c").arg(command).output()?;
    //add command run to history to allow chaining
    let history_file = var("HISTFILE").expect("HISTFILE must be set");
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let history_entry = format!(": {}:0;{}\n", timestamp, command);

    // append command to history file like shell does
    let mut file = OpenOptions::new().append(true).open(history_file)?;

    file.write_all(history_entry.as_bytes())?;

    if !output.stdout.is_empty() {
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(output)
}
