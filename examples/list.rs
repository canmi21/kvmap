/* examples/list.rs */

use kvmap::{Listing, Pathmap};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with a custom path in the user's home directory
    let home = std::env::var("HOME").unwrap();
    let demo_path = format!("{}/.pathmap_demo", home);
    let pm = Pathmap::new().with_base_path(&demo_path);

    // --- Cleanup previous runs if necessary ---
    if std::path::Path::new(&demo_path).exists() {
        std::fs::remove_dir_all(&demo_path)?;
    }
    std::fs::create_dir_all(&demo_path)?;

    println!("--- Initializing Namespaces ---");
    pm.init_ns("words").await?;
    pm.init_ns("users").await?;
    println!("Namespaces created.");

    println!("\n--- Populating Data ---");
    pm.overwrite("words::english.greeting", "Hello").await?;
    pm.overwrite("words::english.farewell", "Goodbye").await?;
    pm.overwrite("words::french.greeting", "Bonjour").await?;
    pm.overwrite("words::config", 42).await?; // A direct value in the namespace

    let user = User {
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
    };
    pm.overwrite("users::profiles.john", &user).await?;
    println!("Data populated.");

    // --- NEW: Listing Functionality ---
    println!("\n--- Testing Listing ---");

    // 1. List all namespaces
    let mut all_ns = pm.list_ns()?;
    println!("All namespaces: {:?}", all_ns);
    // Sort for predictable testing
    all_ns.sort();
    assert_eq!(all_ns, vec!["users".to_string(), "words".to_string()]);

    // 2. List the root of the "words" namespace
    let words_root = pm.list("words").await?;
    println!("Contents of 'words': {:?}", words_root);
    assert_eq!(
        words_root,
        Listing {
            groups: vec!["english".to_string(), "french".to_string()],
            values: vec!["config".to_string()],
        }
    );

    // 3. List a specific group "words::english"
    let words_english = pm.list("words::english").await?;
    println!("Contents of 'words::english': {:?}", words_english);
    assert_eq!(
        words_english,
        Listing {
            groups: vec![],
            values: vec!["farewell".to_string(), "greeting".to_string()],
        }
    );

    // 4. List the root of the "users" namespace
    let users_root = pm.list("users").await?;
    println!("Contents of 'users': {:?}", users_root);
    assert_eq!(
        users_root,
        Listing {
            groups: vec!["profiles".to_string()],
            values: vec![],
        }
    );

    println!("\n--- Test Cleanup ---");
    pm.delete_ns("words").await?;
    pm.delete_ns("users").await?;
    println!("Namespaces deleted.");

    Ok(())
}
