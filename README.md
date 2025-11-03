# Kvmap (formerly Pathmap)

[![Crates.io](https://img.shields.io/crates/v/kvmap.svg)](https://crates.io/crates/kvmap)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://docs.rs/kvmap/badge.svg)](https://docs.rs/kvmap)

Kvmap is a path-driven, namespaced data store for Rust, powered by SQLite. It provides a simple and efficient way to store and retrieve data using a path-like syntax, such as `namespace::group.key`, with support for JSON-serializable values and asynchronous operations.

## Features

- **Path-based Access**: Store and retrieve data using a hierarchical path syntax (`namespace::group.key`).
- **SQLite Backend**: Persistent storage with SQLite, ensuring reliability and performance.
- **Asynchronous API**: Built with `tokio` and `sqlx` for non-blocking operations.
- **Namespace Management**: Create, delete, and manage namespaces with ease.
- **JSON Serialization**: Store and retrieve any JSON-serializable data using `serde`.
- **Background Cleanup**: Automatic database maintenance with customizable intervals.
- **Error Handling**: Comprehensive error handling with `thiserror`.

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
kvmap = "0.1"
```

Ensure you have the required dependencies (`tokio`, `sqlx`, etc.) as specified in the `Cargo.toml` file.

## Usage

Below is a quick example demonstrating how to use Pathmap. For a complete example, see the `examples/demo.rs` file.

```rust
use kvmap::Pathmap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize Pathmap with a custom base path
    let pm = Pathmap::new().with_base_path("/opt/ns");

    // Create a namespace
    pm.init_ns("users").await?;

    // Store a value
    let user = User {
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
    };
    pm.overwrite("users::profiles.john", &user).await?;

    // Retrieve a value
    let retrieved: User = pm.get("users::profiles.john").await?;
    println!("Retrieved: {:?}", retrieved);

    // Check existence
    println!("Exists: {}", pm.exists("users::profiles.john").await?);

    // Delete a value
    pm.delete("users::profiles.john").await?;

    // Clean up namespace
    pm.delete_ns("users").await?;

    Ok(())
}
```

## Project Structure

```
kvmap/
├── examples/
│   └── demo.rs         # Example usage of Pathmap
├── src/
│   ├── db.rs           # SQLite database operations
│   ├── error.rs        # Custom error types
│   └── lib.rs          # Core Pathmap implementation
├── .editorconfig       # Editor configuration
├── .env                # Environment variables
├── .gitattributes      # Git attributes
├── .gitignore          # Git ignore file
├── Cargo.lock          # Dependency lock file
├── Cargo.toml          # Project manifest
├── clay-config.json    # Configuration file
└── LICENSE             # MIT License
```

## API Overview

- **`Pathmap::new()`**: Creates a new Pathmap instance with the default base path (`/opt/pathmap/`).
- **`with_base_path(path)`**: Overrides the default base path.
- **`init_ns(ns)`**: Initializes a new namespace, creating a SQLite file.
- **`delete_ns(ns)`**: Deletes a namespace and its SQLite file.
- **`get<T>(path)`**: Retrieves a JSON-deserializable value from a path.
- **`set<T>(path, value)`**: Sets a value at a path (fails if the key exists).
- **`overwrite<T>(path, value)`**: Sets or updates a value at a path.
- **`delete(path)`**: Deletes a value at a path.
- **`exists(path)`**: Checks if a namespace, group, or value exists.
- **`manual_cleanup(ns)`**: Triggers a manual database cleanup (VACUUM).
- **`start_background_cleanup(interval, timeout)`**: Starts automatic cleanup for idle namespaces.

## Dependencies

Pathmap relies on the following Rust crates:

- `tokio = { version = "1", features = ["full"] }`
- `fancy-log = "0.1"`
- `sqlx = { version = "0.8", features = ["runtime-tokio-native-tls", "sqlite"] }`
- `thiserror = "2"`
- `shellexpand = "3"`
- `serde = { version = "1.0", features = ["derive"] }`
- `serde_json = "1"`

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please submit issues or pull requests to the [GitHub repository](https://github.com/canmi21/pathmap).

## Contact

For questions or feedback, open an issue on the [GitHub repository](https://github.com/canmi21/pathmap).