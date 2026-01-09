# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

cllmi is a Rust CLI tool that corrects failed shell commands using the Claude API. It captures the last command from shell history, re-runs it to get error output, sends it to Claude for correction, and presents the fixed command for execution or clipboard copy.

## Build and Run Commands

```bash
# Build and install
./build.sh                    # Or: cargo build --release && cargo install --path .

# Run
cllmi                         # Basic mode - fixes last failed command
cllmi -j                      # Request justification
cllmi -c "context here"       # Add context about your intent
cllmi -g "command idea"       # Guide mode - generate command without history lookup

# Development
cargo build                   # Debug build
cargo run -- -j               # Run with args
```

## Environment Requirements

- `CLAUDE_API_KEY` - Required for API access
- `HISTFILE` - Shell history file path (typically set by zsh/bash)

## Architecture

Three source files in `src/`:

- **main.rs** - Entry point, CLI parsing (clap), async orchestration, Claude API calls via reqwest, clipboard integration
- **interface.rs** - `Request`/`Response` structs for API communication, shell history access via `HISTFILE`, command execution with history logging for chaining
- **sysprompt.rs** - Static system prompt generation with OS context

### Communication Protocol

The tool uses XML tags for Claude communication:
- Request tags: `<command>`, `<output>`, `<context>`, `<justification_requested/>`
- Response tags: `<fixed_command>`, `<justification>`

### Key Design Decisions

- Single-threaded Tokio runtime (`current_thread` flavor)
- System prompt initialized once via `lazy_static!`
- Executed commands are appended to shell history with timestamps to enable command chaining
- Validation happens at payload construction time (`Request::to_payload()`)
