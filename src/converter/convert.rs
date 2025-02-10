use std::env;
use std::fs;
use std::error::Error; // Import the Error trait
use std::process; // For cleaner error handling
use std::path::Path;
use colored::*;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    //getting the parameters from the terminal
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", format!("Usage: {} <file_path>", args[0]).yellow());
        process::exit(1); // Exit with a non-zero code to indicate an error
    }
    let file_path = &args[1];
    let path = Path::new(file_path);

    // if let checks if the path has an extension and when it does it holds it
    if let Some(extension) = path.extension() {
        if extension == "mod" {
            let contents = read_file(file_path)?; // Use ? for error propagation

            // More efficient way to write the modified contents:
            fs::write(file_path, search_and_create_socket(&contents))?; // Pass a reference

        } else {
            eprintln!("{}", "Error: Only *.mod files are accepted".red());
            process::exit(1);
        }
    } else {
        eprintln!("{}", "Error: File has no extension.".red());
        process::exit(1);
    }

    Ok(())
}

fn read_file(file_path: &String)->Result<String, Box<dyn Error>>{
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            Ok(contents) // Return Ok(()) to indicate success
        }
        Err(e) => {
            eprintln!("{}", format!("Error reading file '{}': {}", file_path, e).red());
            Err(Box::new(e)) // Box the error to satisfy the Result type
        }
    }
}

fn search_and_create_socket(contents: &String) -> String{
    //we have to check if a socket is already in the file and initialized
    //if not we have to create a socket on a specifc ip adress and port
    //manipulate the string so that we add a socket in Rapid-Code Style
    //return the new string
    //currently the socket will be created as the first thing
    if contents.contains("VAR socketdev my_socket") 
    && contents.contains("SocketCreate my_socket") 
    && contents.contains("SocketConnect my_socket, \"192.168.0.1\", 1025")
    {
        contents.to_string()
    }
    else{
        format!("VAR socketdev my_socket;
        \nSocketCreate my_socket;
        \nSocketConnect my_socket, \"192.168.0.1\", 1025;
        \n{}", contents)
    }
}