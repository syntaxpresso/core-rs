use clap::Parser;
use syntaxpresso_core::commands::Commands;
use syntaxpresso_core::responses::error_response::ErrorResponse;

#[derive(Parser)]
#[command(name = "syntaxpresso-core")]
#[command(
  about = "A standalone Rust-based CLI backend for IDE plugins that provides advanced Java code generation and manipulation capabilities using Tree-Sitter."
)]
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
