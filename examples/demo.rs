/* examples/demo.rs */

use kvmap::Pathmap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize with a custom path in the user's home directory
    let pm = Pathmap::new().with_base_path("/opt/ns");

    // Cleanup previous runs if necessary
    if pm.exists("words").await? {
        pm.delete_ns("words").await?;
    }

    // 1. Initialize a namespace
    println!("Initializing namespace 'words'...");
    pm.init_ns("words").await?;
    println!("Namespace 'words' created: {}", pm.exists("words").await?);

    // 2. Add a value (and implicitly the group)
    println!("\nOverwriting a value 'words::english.greeting'...");
    pm.overwrite("words::english.greeting", "Hello, Pathmap!")
        .await?;

    // 3. Get the value
    let greeting: String = pm.get("words::english.greeting").await?;
    println!("Retrieved value: {}", greeting);

    // 4. Set a value (will fail if it exists)
    println!("\nTrying to 'set' an existing value (should fail)...");
    let set_result = pm.set("words::english.greeting", "New Greeting").await;
    assert!(set_result.is_err());
    println!("'set' failed as expected: {:?}", set_result.err());

    // 5. Working with complex data (structs)
    let user = User {
        name: "John Doe".to_string(),
        email: "john.doe@example.com".to_string(),
    };
    println!("\nOverwriting a complex value 'words::users.john'...");
    pm.overwrite("words::users.john", &user).await?;

    let retrieved_user: User = pm.get("words::users.john").await?;
    println!("Retrieved user: {:?}", retrieved_user);
    assert_eq!(user, retrieved_user);

    // 6. Check for existence
    println!("\nChecking existence...");
    println!("'words' exists: {}", pm.exists("words").await?);
    println!(
        "'words::english' exists: {}",
        pm.exists("words::english").await?
    );
    println!(
        "'words::english.greeting' exists: {}",
        pm.exists("words::english.greeting").await?
    );
    println!(
        "'words::french' exists: {}",
        pm.exists("words::french").await?
    );

    // 7. Delete a value
    println!("\nDeleting value 'words::english.greeting'...");
    pm.delete("words::english.greeting").await?;
    println!(
        "'words::english.greeting' exists: {}",
        pm.exists("words::english.greeting").await?
    );

    // 8. Manual cleanup
    println!("\nTriggering manual cleanup for 'words' namespace...");
    pm.manual_cleanup("words").await?;
    println!("Manual cleanup finished.");

    // 9. Delete namespace
    println!("\nDeleting namespace 'words'...");
    pm.delete_ns("words").await?;
    println!("Namespace 'words' exists: {}", pm.exists("words").await?);

    // Example of background cleanup (won't do much in this short demo)
    // pm.start_background_cleanup(Duration::from_secs(10), Duration::from_secs(30));
    // tokio::time::sleep(Duration::from_secs(40)).await; // Keep app running to see cleanup

    Ok(())
}
