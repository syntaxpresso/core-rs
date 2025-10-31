pub mod commands;
pub mod common;
pub mod responses;

use clap::Parser;
use commands::Commands;

use crate::responses::error_response::ErrorResponse;

#[derive(Parser)]
#[command(name = "core-rs")]
#[command(about = "A CLI application that handles commands and returns JSON responses")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();

    match cli.command.execute() {
        Ok(json) => println!("{}", json),
        Err(e) => {
            let error_response =
                ErrorResponse { error: "execution_error".to_string(), message: e.to_string() };
            match serde_json::to_string_pretty(&error_response) {
                Ok(error_json) => println!("{}", error_json),
                Err(_) => eprintln!("Critical error: Failed to serialize error response"),
            }
        }
    }
}
