use std::env;
use std::fs;
use std::error::Error; // Import the Error trait
use std::process; // For cleaner error handling

fn main(){
    let contents = read_file();
    println!("{:?}", contents);
}

fn read_file()->Result<String, Box<dyn Error>>{
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]); // Print to stderr for errors
        process::exit(1); // Exit with a non-zero code to indicate an error
    }

    let file_path = &args[1];

    match fs::read_to_string(file_path) {
        Ok(contents) => {
            Ok(contents) // Return Ok(()) to indicate success
        }
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            Err(Box::new(e)) // Box the error to satisfy the Result type
        }
    }
}