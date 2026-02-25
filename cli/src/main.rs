use arboard::Clipboard;
use clap::{Parser, Subcommand};
use hush_core::crypto::{decrypt, encrypt};
use keyring::Entry;
use rand::distr::{Alphanumeric, SampleString};
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, io, thread, time::Duration};

const SERVICE_NAME: &str = "hush-secrets-manager";
const ACCOUNT_NAME: &str = "master-key";

#[derive(Parser)]
#[command(name = "hush")]
#[command(about = "A Zero-Knowledge Secrets Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bind this device with your Master Key
    Login,

    /// Securely wipe Master Key from device hardware
    Logout,

    /// Create a new 32-character secure random key
    Generate,

    /// Print a decrypted secret to your terminal
    Get {
        /// The name of the secret to retrieve
        name: String,
    },

    /// Copy a secret to your clipboard (auto-clears in 15s)
    Copy {
        /// The name of the secret to copy
        name: String,
    },

    /// Encrypt and store a new secret in the vault
    Set {
        /// The name of the secret
        name: String,

        /// The plaintext value to encrypt
        value: String,
    },
    ExportKey,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Clap safely guarantees our arguments are present,
    // so we can directly match on the enums!
    match cli.command {
        Commands::Login => handle_login().await,
        Commands::Logout => handle_logout().await,
        Commands::Generate => handle_generate().await,
        Commands::Get { name } => handle_get(&name).await,
        Commands::Copy { name } => handle_copy(&name).await,
        Commands::Set { name, value } => handle_set(&name, &value).await,
        Commands::ExportKey => handle_export_key().await,
    }
}

fn get_credentials_path() -> PathBuf {
    let mut path = dirs::home_dir().expect("Could not find home directory");
    path.push(".hush");
    fs::create_dir_all(&path).expect("Failed to create .hush directory");
    path.push("credentials");
    path
}

async fn handle_login() {
    let key = rpassword::prompt_password("Enter 32-char Master Key (input hidden): ").unwrap();
    let key = key.trim();

    if key.len() == 32 {
        let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).unwrap();

        // Try OS Keyring first
        if entry.set_password(key).is_ok() {
            println!("✅ Device Bound. Key secured by OS hardware.");
        } else {
            // FALLBACK: Secure local file (WSL/Linux Server mode)
            let path = get_credentials_path();
            let mut file = File::create(&path).expect("Failed to create credentials file");

            // Lock permissions to 600 (Read/Write for owner ONLY)
            let mut perms = file.metadata().unwrap().permissions();
            perms.set_mode(0o600);
            file.set_permissions(perms)
                .expect("Failed to set secure permissions");

            file.write_all(key.as_bytes()).expect("Failed to write key");
            println!(
                "✅ Device Bound. Key saved to securely locked file at {:?}",
                path
            );
        }
    } else {
        println!("❌ Error: Key must be exactly 32 characters.");
    }
}

async fn handle_logout() {
    // 1. Clear Keyring
    let entry = Entry::new(SERVICE_NAME, ACCOUNT_NAME).unwrap();
    let _ = entry.delete_password();

    // 2. Clear Local File
    let path = get_credentials_path();
    if path.exists() {
        let _ = fs::remove_file(path);
    }

    println!("👋 Logged out. Master key removed from device.");
}

async fn handle_generate() {
    // rand 0.10.0 uses the new distr module and SampleString trait
    let secret = Alphanumeric.sample_string(&mut rand::rng(), 32);

    println!("Here is a secure, 32-character string:");
    println!("----------------------------------");
    println!("{}", secret);
    println!("----------------------------------");
    println!("You can use this for your Master Key or for a new database password.");
}

async fn handle_copy(name: &str) {
    let key = get_master_key();
    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://localhost:3000/api/secrets/{}", name))
        .send()
        .await
        .expect("Request failed");

    if res.status() == 404 {
        println!("❌ Secret '{}' not found", name);
        return;
    }
    let body: serde_json::Value = res.json().await.expect("JSON error");

    let ciphertext: Vec<u8> = body["secret"]["ciphertext"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    let nonce: Vec<u8> = body["secret"]["nonce"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();

    match decrypt(&ciphertext, &nonce, &key) {
        Ok(val) => {
            let mut cb = Clipboard::new().expect("Failed to open clipboard");
            cb.set_text(val.clone()).unwrap();
            println!("📋 Secret '{}' copied!", name);
            println!("⚠️ Note: This does not clear OS Clipboard History (like Win+V).");

            print!("🧹 Clearing in: ");
            for i in (1..=15).rev() {
                print!("{}.. ", i);
                io::stdout().flush().unwrap();
                thread::sleep(Duration::from_secs(1));
            }

            let mut cb_inner = Clipboard::new().unwrap();
            if cb_inner.get_text().unwrap_or_default() == val {
                let _ = cb_inner.set_text("");
                println!("\n✅ Clipboard cleared securely.");
            } else {
                println!("\nℹ️ Clipboard overwritten by another app. Skipping clear.");
            }
        }
        Err(_) => println!("❌ Decryption failed. Wrong Master Key?"),
    }
}

async fn handle_get(name: &str) {
    let key = get_master_key();
    let client = reqwest::Client::new();
    let res = client
        .get(format!("http://localhost:3000/secrets/{}", name))
        .send()
        .await
        .expect("Request failed");

    if res.status() == 404 {
        println!("❌ Secret not found");
        return;
    }
    let body: serde_json::Value = res.json().await.expect("JSON error");

    let ciphertext: Vec<u8> = body["secret"]["ciphertext"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();
    let nonce: Vec<u8> = body["secret"]["nonce"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();

    match decrypt(&ciphertext, &nonce, &key) {
        Ok(val) => println!("{}: {}", name, val),
        Err(_) => println!("❌ Decryption failed."),
    }
}

async fn handle_set(name: &str, value: &str) {
    let key = get_master_key();
    let secret = encrypt(value, &key).expect("Encryption failed");

    let payload = serde_json::json!({ "name": name, "secret": secret });
    let res = reqwest::Client::new()
        .post("http://localhost:3000/secrets") // <-- Removed /api
        .json(&payload)
        .send()
        .await
        .expect("Request failed");

    if res.status().is_success() {
        println!("✅ Stored.");
    } else {
        // Now it will scream at you if it fails!
        println!("❌ Server Error: {}", res.status());
    }
}

fn get_master_key() -> [u8; 32] {
    println!("🔍 DEBUG: Attempting to find Master Key...");

    // 1. Try OS Keyring
    if let Ok(entry) = Entry::new(SERVICE_NAME, ACCOUNT_NAME) {
        if let Ok(key_str) = entry.get_password() {
            println!("✅ DEBUG: Found key in OS Keyring.");
            let mut key = [0u8; 32];
            key.copy_from_slice(key_str.as_bytes());
            return key;
        } else {
            println!("ℹ️ DEBUG: OS Keyring accessible, but no key found.");
        }
    } else {
        println!("ℹ️ DEBUG: OS Keyring not accessible (Expected on WSL).");
    }

    // 2. Try Secure Local File
    let path = get_credentials_path();
    println!("🔍 DEBUG: Checking local file at {:?}", path);
    if path.exists() {
        if let Ok(key_str) = fs::read_to_string(&path) {
            let key_str = key_str.trim();
            println!(
                "🔍 DEBUG: File contents length after trim: {}",
                key_str.len()
            );
            if key_str.len() == 32 {
                println!("✅ DEBUG: Found valid key in ~/.hush/credentials.");
                let mut key = [0u8; 32];
                key.copy_from_slice(key_str.as_bytes());
                return key;
            } else {
                println!(
                    "❌ DEBUG: File exists, but key length is {} (Expected 32)",
                    key_str.len()
                );
            }
        } else {
            println!("❌ DEBUG: File exists but could not be read (permissions?)");
        }
    } else {
        println!("ℹ️ DEBUG: Local file does not exist.");
    }

    // 3. Try Env Var
    println!("🔍 DEBUG: Checking HUSH_MASTER_KEY environment variable...");
    if let Ok(key_str) = env::var("HUSH_MASTER_KEY") {
        let key_str = key_str.trim();
        if key_str.len() == 32 {
            println!("✅ DEBUG: Found valid key in environment variable.");
            let mut key = [0u8; 32];
            key.copy_from_slice(key_str.as_bytes());
            return key;
        } else {
            println!(
                "❌ DEBUG: Env var exists, but length is {} (Expected 32)",
                key_str.len()
            );
        }
    }

    println!("❌ Vault is locked on this device. Run 'hush login' first.");
    std::process::exit(1);
}

async fn handle_export_key() {
    // get_master_key() already handles checking the Keyring,
    // the ~/.hush/credentials file, and the env variable for us!
    let key = get_master_key();
    let key_str = std::str::from_utf8(&key).expect("Invalid UTF-8 sequence");

    println!("🔑 Master Key Found!");
    println!("Copy the string below and paste it into http://localhost:3000");
    println!("----------------------------------------------------------------");
    println!("{}", key_str);
    println!("----------------------------------------------------------------");
    println!("⚠️  Tip: Run `clear` in your terminal afterward to hide it from screen-sharing.");
}
