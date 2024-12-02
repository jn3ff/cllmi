use std::env::var;

use clap::{command, Parser};
use clipboard::{ClipboardContext, ClipboardProvider};
use interface::{get_last_command, run_command_with_history, Request};
use sysprompt::get_sys_prompt;

mod interface;
mod sysprompt;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Model to use. This gets sent straight to the api, so if you override, make sure it's a valid model string
    /// I also have not tested it at all with anything other than the default
    #[arg(short, long, default_value_t = String::from("claude-3-5-sonnet-20241022"))]
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
        )
        .expect("could not construct guide command");
    } else {
        req = get_last_command().expect("last command not present");
    }

    if !args.context.is_empty() {
        req.add_context(args.context);
    }
    if args.justify {
        req.compel_justification();
    }

    let res = ask_claude(req, args.model, key).await.unwrap();

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
) -> Result<interface::Response, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let payload = serde_json::json!({
        "model": model,
        "max_tokens": 1024,
        "system": get_sys_prompt(), //TODO: make sys prompt static
        "messages": [
            {"role": "user", "content": req.to_payload()}
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

    let response_text = response.text().await?;
    let response = interface::Response::from_string(response_text)
        .expect("failed to deserialize response from the api");
    Ok(response)
}
