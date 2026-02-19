use clap::{Parser, Subcommand};
use reqwest::Client;

#[derive(Parser)]
#[command(name = "hush")]
#[command(version = "1.0")]
#[command(about = "A zero-knowledge secrets manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set {
        /// The name of the secret (e.g., stripe-api-key)
        name: String,
        /// The plaintext value to encrypt
        value: String,
    },
    /// Retrieve and decrypt a secret
    Get {
        /// The name of the secret to fetch
        name: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let _client = Client::new();
    let _backend_url = "http://localhost:3000";

    match &cli.command {
        Commands::Set { name, value } => {
            println!("Preparing to encrypt secret: {}", name);
            println!("(Plaintext value received: {})", value);
            // TODO: Implement CLI-2 logic here
        }
        Commands::Get { name } => {
            println!("Preparing to fetch secret: {}", name);
            // TODO: Implement CLI-3 logic here
        }
    }
}
