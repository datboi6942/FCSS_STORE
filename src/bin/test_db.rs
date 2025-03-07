use std::fs::File;
use std::io::prelude::*;

fn main() -> std::io::Result<()> {
    // Path to the database file
    let path = "/home/john/FCSS_Store/FCSS_STORE/data/secure_store.db";
    
    println!("Testing if we can write to: {}", path);
    
    // Try to open the file for writing
    let mut file = File::create(path)?;
    
    // Write something to it
    file.write_all(b"SQLite format 3\0")?;
    
    println!("Successfully wrote to the database file!");
    
    Ok(())
} 