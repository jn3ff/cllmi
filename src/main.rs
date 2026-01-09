use std::env::var;

use clap::{command, Parser};
use clipboard::{ClipboardContext, ClipboardProvider};
use interface::{get_last_command, run_command_with_history, Request};
use lazy_static::lazy_static;
use sysprompt::get_sys_prompt;

mod interface;
mod sysprompt;

lazy_static! {
    static ref SYS_PROMPT: String = get_sys_prompt();
}

fn resolve_model(model: &str) -> String {
    match model.to_lowercase().as_str() {
        "sonnet" => String::from("claude-sonnet-4-5-20250929"),
        "opus" => String::from("claude-opus-4-5-20251101"),
        _ => String::from(model), // pass through full model strings
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Model to use. Shorthands: "opus" (default), "sonnet". Full model strings also accepted.
    #[arg(short, long, default_value_t = String::from("opus"))]
    model: String,
    /// Any contextual information about the goal of your command, to be sent to the api so it can make a better decision
    #[arg(short, long, default_value_t = String::from(""))]
    context: String,
    /// use flag to compel model to give you a justification for its selected command
    #[arg(short, long, default_value_t = false)]
    justify: bool,

    /// avoids looking up last command, just put in the idea for the command here
    #[arg(short, long, default_value_t = String::from(""))]
    guide: String,

    /// print raw model response for debugging
    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let key: String = var("CLAUDE_API_KEY").expect("must have a key set for CLAUDE_API_KEY");

    let args = Args::parse();
    let mut req;
    if !args.guide.is_empty() {
        if args.context.is_empty() {
            println!("No context provided, expect poorer results.")
        }
        req = Request::new(
            args.guide.as_str(),
            None,
            Some("Please infer desired usage from the provided command"),
            false,
        );
    } else {
        req = get_last_command().expect("last command not present");
    }

    if !args.context.is_empty() {
        req.add_context(args.context);
    }
    if args.justify {
        req.compel_justification();
    }

    let model = resolve_model(&args.model);
    let res = ask_claude(req, model, key, args.debug).await.unwrap();

    println!("\nSuggested command: {}", res.command);
    if let Some(justification) = res.justification {
        println!("\nJustification: {}", justification);
    }

    println!(
        "\nPress Enter to execute the command, 'c' to copy to clipboard, or Ctrl+C to cancel..."
    );
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    // TODO: use key events. c -> enter is annoying
    match input.trim() {
        "" => match run_command_with_history(&res.command) {
            Ok(output) => {
                if !output.status.success() {
                    std::process::exit(output.status.code().unwrap_or(1));
                }
            }
            Err(e) => {
                eprintln!("Could not successfully run command from cli output");
                panic!("{:?}", e);
            }
        },
        "c" => {
            let mut ctx: ClipboardContext =
                ClipboardProvider::new().expect("Failed to initialize clipboard");
            ctx.set_contents(res.command.clone())
                .expect("Failed to copy to clipboard");
            println!("Command copied to clipboard!");
            std::process::exit(0);
        }
        _ => {
            println!("Command execution cancelled");
            std::process::exit(1);
        }
    }
}

async fn ask_claude(
    req: Request,
    model: String,
    key: String,
    debug: bool,
) -> Result<interface::Response, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "system": *SYS_PROMPT,
        "messages": [
            {"role": "user", "content": req.to_payload().expect("invalid payload")}
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;
    if debug {
        eprintln!("\n[DEBUG] Raw API response:\n{}\n", response_text);
    }

    if !status.is_success() {
        let err = interface::Response::from_api_error(&response_text);
        panic!("API error: {:?}", err);
    }

    let response = interface::Response::from_string(response_text);
    match response {
        Ok(r) => Ok(r),
        Err(e) => {
            panic!("Failed to parse response: {:?}", e);
        }
    }
}
