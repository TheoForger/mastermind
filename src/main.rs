use std::env;
use std::path::PathBuf;

use clap::Parser;
use dotenv::dotenv;

use api_handlers::chat_completions::*;
use api_handlers::language_models::*;
use mastermind::*;

/// Mastermind - An LLM-powered CLI tool to help you be a better spymaster in Codenames
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Get available language json_models from API
    #[arg(short, long = "get-models")]
    get: bool,

    /// Specify a language model
    #[arg(short, long = "set-model")]
    model: Option<String>,

    /// Path to a file containing words to link together - the words from your team
    #[arg(required_unless_present = "get")]
    to_link: Option<PathBuf>,

    /// Path to a file containing words to avoid - opponent's words, neutral words, and the assassin word
    #[arg(required_unless_present = "get")]
    to_avoid: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read arguments and environment variables
    let args = Args::parse();
    dotenv().ok();

    // Get all model IDs for future reference
    let model_ids = get_model_ids_from_api().await?;

    // If -g is set, call the models API endpoint instead
    if args.get {
        println!("{}", model_ids.join("\n"));
        return Ok(());
    }

    // If -m is set, use a preferred language model
    // Otherwise, use the default
    let model_id = match args.model {
        Some(id) => id,
        None => env::var("DEFAULT_MODEL_ID").map_err(|_| {
            "Could not read environment variable: DEFAULT_MODEL_ID. Use -m to specify a language model".to_string()
        })?,
    };

    // Abort the program if the chosen model is not valid
    if !model_ids.contains(&model_id) {
        return Err(format!(
            "{} is not a valid language model from your provider",
            model_id
        )
        .into());
    }

    // Attempt to read words from the two files
    let link_words = read_words_from_file(args.to_link.unwrap()).map_err(|e| e.to_string())?;
    let avoid_words = read_words_from_file(args.to_avoid.unwrap()).map_err(|e| e.to_string())?;

    // Get clues from API
    let clue_collection = get_clue_collection_from_api(link_words, avoid_words, &model_id).await?;

    // Output
    if clue_collection.is_empty() {
        println!("The language model didn't return any useful clues. Maybe try again?");
    } else {
        clue_collection.display();
    }

    Ok(())
}
