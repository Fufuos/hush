# Hush - Zero-Knowledge Secrets Manager

Hush is a full-stack, end-to-end encrypted secrets manager built entirely in Rust. It utilizes a **Zero-Knowledge Architecture** to ensure that sensitive data (like API keys, database passwords, and environment variables) is encrypted on the client's machine before it ever touches the network.

The remote server acts as a blind storage locker—it holds the ciphertext but has no ability to read the underlying data.

## Monorepo Architecture

This project is structured as a Cargo workspace with three distinct crates to strictly separate concerns:

- **`hush-core` (The Engine):** A mathematically rigorous cryptographic library implementing **AES-256-GCM**. It provides authenticated encryption, ensuring data is both unreadable and tamper-proof.
- **`backend` (The API):** A high-performance, asynchronous REST API built with **Axum** and **Tokio**. It uses **SQLx** for compile-time verified queries against a **SQLite** database to store the encrypted payloads.
- **`cli` (The Client):** A developer-friendly terminal interface built with **Clap**. It handles local key management, transparently encrypts user input, and synchronizes the "noise" with the backend.

## The "Zero-Knowledge" Guarantee

1. **Local Encryption:** When you run `hush set`, the CLI uses your local 32-byte Master Key to encrypt the plaintext.
2. **Blind Storage:** The CLI sends only the resulting ciphertext and a randomized nonce to the Axum backend. The backend **never** sees your Master Key or your plaintext.
3. **Local Decryption:** When you run `hush get`, the CLI downloads the ciphertext and decrypts it locally. If a database breach occurs, the attacker only gets mathematically useless noise.

## Getting Started

### Prerequisites

- [Rust toolchain](https://rustup.rs/)
- SQLx CLI (`cargo install sqlx-cli --no-default-features --features rustls,sqlite`)

### 1. Database Setup

Create the SQLite database and run the migrations:

```bash
cd backend
sqlx database create
sqlx migrate run
```

### 2. Environment Variables

Create a .env file in the root of the workspace:

```
DATABASE_URL="sqlite:backend/hush.db"
```

### 3. Run the Server

Start the Axum backend in one terminal window:

```bash
cargo run -p backend
```

### 4. Use the CLI

In a second terminal, provide your 32-byte master key and manage your secrets:

Store a secret:

```bash
HUSH_MASTER_KEY="this-is-exactly-32-chars-long-12" cargo run -p cli -- set stripe-key sk_test_12345
```

Retrieve a secret:

```bash
HUSH_MASTER_KEY="this-is-exactly-32-chars-long-12" cargo run -p cli -- get stripe-key
```

## Testing

The cryptographic core is fully unit-tested to guarantee safe round-trip encryption, safe failures on incorrect keys, and strict integrity checks against ciphertext tampering.

```bash
cargo test -p hush-core
```
