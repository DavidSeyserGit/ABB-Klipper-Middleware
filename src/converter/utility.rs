use std::path::Path;
use std::fs;
use std::error::Error; // Import the Error trait
use regex::Regex;

pub fn read_file(file_path: &Path)->Result<String, Box<dyn Error>>{
    match fs::read_to_string(file_path) {
        Ok(contents) => {
            Ok(contents) // Return Ok(()) to indicate success
        }
        Err(e) => {
            eprintln!("Error reading file '{:?}': {}", file_path, e);
            Err(Box::new(e)) // Box the error to satisfy the Result type
        }
    }
}

pub fn search_and_create_socket(contents: &String) -> String{
//we have to check if a socket is already in the file and initialized
//if not we have to create a socket on a specifc ip adress and port
//manipulate the string so that we add a socket in Rapid-Code Style
//return the new string
//currently the socket will be created as the first thing -> might need to change it so that this is after the Module Name
if contents.contains("VAR socketdev my_socket") 
&& contents.contains("SocketCreate my_socket") 
&& contents.contains("SocketConnect my_socket, \"192.168.0.1\", 1025")
{
    contents.to_string() //if a socket is already in the program we dont have to do anything
}
else{
    //otherwise we create the sockets ourselfs
    format!("VAR socketdev my_socket;
    \nSocketCreate my_socket;
    \nSocketConnect my_socket, \"192.168.0.1\", 1025;
    \n{}", contents)
}
}

pub fn replace_call_extruder_with_socket_send(contents: &String)->String{
let re = Regex::new(r"Extruder(\d+)").unwrap();
let mut new_contents = String::new();

for lines in contents.lines(){
    if let Some(captures) = re.captures(lines){
        let number_str = captures.get(1).unwrap().as_str(); //get the number (match group1)
        let number = number_str.parse::<f32>().unwrap()/1000.00; // get it to a number
        new_contents.push_str(&format!("    SocketSend my_socket \\Str '{}';\n", number));
    }
    else{
        new_contents.push_str(lines); // Append the original line
        new_contents.push_str("\n"); // Add the newline back
    }
}
new_contents
}


pub fn replace_setrpm_with_socket_send(_contents: &String)->String{

todo!();
}