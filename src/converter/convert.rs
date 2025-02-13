use std::env;
use std::fs;
use std::error::Error; // Import the Error trait
use std::process; // For cleaner error handling
use std::path::Path;
use colored::*;
mod utility;

fn main() -> Result<(), Box<dyn std::error::Error>>{
    //getting the parameters from the terminal
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", format!("Usage: {} <file_path>", args[0]).yellow());
        process::exit(1); // Exit with a non-zero code to indicate an error
    }
    let path = Path::new(&args[1]);
    
    if !path.is_dir(){
        process_file(path)?;
    }else{
        process_directory(path)?;
    }
    Ok(())
}

fn process_directory(path: &Path) -> Result<(), Box<dyn Error>>{
    for entries in fs::read_dir(path)?{
        let entries = entries?;
        let entries_path = entries.path();
        println!("{}", format!("Processing File: {}", entries_path.to_str().unwrap()).green());

        // if let checks if the path has an extension and when it does it holds it
        if let Some(extension) = entries_path.extension() {
            if extension == "mod" {
                let mut contents = utility::read_file(&entries_path)?;

                //creation of the socket should happen only once as the other files are loaded in dynamically
                if let Some(file_name) = entries_path.file_name().and_then(|name| name.to_str()) {
                    if file_name == "main.mod" {
                        contents = utility::search_and_create_socket(&contents);
                    }
                }
                contents = utility::replace_call_extruder_with_socket_send(&contents);
                contents = utility::replace_setrpm_with_socket_send(&contents);
                contents = utility::replace_m_code_with_socket_send(&contents);
                fs::write(entries_path, contents)?; // Pass a reference

            } else {
                eprintln!("{}", "Error: Only *.mod files are accepted".red());
                process::exit(1);
            }
        } else {
            eprintln!("{}", "Error: File has no extension.".red());
        }
    }
    Ok(())
}

fn process_file(path: &Path)->Result<(), Box<dyn Error>>{
    //this means we only have one file and not a directory where i wanna replace the contents of
    let mut contents = utility::read_file(&path)?;
    contents = utility::search_and_create_socket(&contents);
    contents = utility::replace_call_extruder_with_socket_send(&contents);
    contents = utility::replace_setrpm_with_socket_send(&contents);
    contents = utility::replace_m_code_with_socket_send(&contents);
    fs::write(path, contents)?; // Pass a reference
    Ok(())
}