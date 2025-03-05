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

//needs to be rewritten to be not the first thing but after the Module declaration
pub fn search_and_create_socket(contents: &String, _postprocess: &String) -> String {
    if contents.contains("VAR socketdev my_socket")
        && contents.contains("SocketCreate my_socket")
        && contents.contains("SocketConnect my_socket, \"10.0.0.10\", 1234")
    {
        return contents.to_string(); // Return immediately if socket code is already present
    }

    let mut modified_contents = String::new();
    let mut module_found = false;
    let mut proc_found = false;

    for line in contents.lines() {
        modified_contents.push_str(line); //push every line into the string
        modified_contents.push('\n'); //make a linebreak

        if line.contains("MODULE") && !module_found { // Only process the first MODULE line
            module_found = true;
            modified_contents.push_str("VAR socketdev my_socket;\n"); // Add variable declaration *after* module
        }

        if line.contains("PROC") && !proc_found {  // Only process the first PROC line
            proc_found = true;
            modified_contents.push_str("\tSocketCreate my_socket;\n\tSocketConnect my_socket, \"10.0.0.10\", 1234;\n"); // Insert create/connect
        }
    }

    if !module_found {
        return "Error: Could not find the MODULE line.".to_string();
    }
    if !proc_found {
        return "Error: Could not find the PROC main_RoboDK() line.".to_string();
    }

    modified_contents
}

pub fn replace_call_extruder_with_socket_send(contents: &String, postprocess: &String) -> String {
    let mut new_contents = String::new();

    if postprocess == "rapid" {
        let re = Regex::new(r"Extruder(\d+)").unwrap();

        for line in contents.lines() {
            if let Some(captures) = re.captures(line) {
                let number_str = captures.get(1).unwrap().as_str();
                let factor = if postprocess == "rapid" { 100000.00 } else { 1.00 };
                let number = number_str.parse::<f32>().unwrap() / factor;
                new_contents.push_str(&format!("    SocketSend my_socket \\Str := \"E{}\";\n", number));
            } else {
                new_contents.push_str(line);
                new_contents.push_str("\n");
            }
        }
    } else { // Handle MoveL and E-value calculation
        let move_regex = Regex::new(r"MoveL \[\[([\d\.\-E]+),([\d\.\-E]+),([\d\.\-E]+)\],\[[\d\.\-E]+,[\d\.\-E]+,[\d\.\-E]+,[\d\.\-E]+],\[[\d,-]+],\[[\dE\+\-]+,[\dE\+\-]+,[\dE\+\-]+,[\dE\+\-]+,[\dE\+\-]+,[\dE\+\-]+]\],(v\d+),.*").unwrap();
        let speed_regex = Regex::new(r"ExtruderSpeed\s*([\d\.]+)").unwrap();

        let mut last_position: Option<[f64; 3]> = None;
        let mut extruder_speed: Option<f64> = None;
        let mut total_e_value: f64 = 0.0; // Initialize total E-value

        for line in contents.lines() {
            if let Some(captures) = move_regex.captures(line) {
                let x: f64 = captures[1].parse().unwrap();
                let y: f64 = captures[2].parse().unwrap();
                let z: f64 = captures[3].parse().unwrap();
                let position = [x, y, z];
                let velocity_str = captures.get(4).unwrap().as_str();

                new_contents.push_str(line);
                new_contents.push_str("\n");

                if let Some(last_pos) = last_position {
                    let distance = ((position[0] - last_pos[0]).powi(2) + (position[1] - last_pos[1]).powi(2) + (position[2] - last_pos[2]).powi(2)).sqrt();

                    if let Some(speed) = extruder_speed {
                        let velocity: f64 = velocity_str[1..].parse().unwrap_or(0.0);

                        if velocity > 0.0 {
                            let _time = distance / velocity;
                            let e_value = distance * speed * 0.05;
                            total_e_value += e_value; // Add to the total E-value
                            let formatted_e_value = format!("{:.2}", total_e_value); // Format to 2 decimal places
                            new_contents.push_str(&format!("    SocketSend my_socket \\Str := \"E{}\";\n", total_e_value)); // Send the cumulative E value
                        } else {
                            new_contents.push_str("    !Warning: Velocity is zero or invalid for this MoveL command.\n");
                        }
                    } else {
                        new_contents.push_str("    !Warning: Extruder speed not yet defined.\n");
                    }
                }

                last_position = Some(position);
            } else if let Some(captures) = speed_regex.captures(line) {
                extruder_speed = Some(captures[1].parse().unwrap());
                new_contents.push_str("\n");
                new_contents.push_str(&format!("    !Extruder Speed: {}\n", extruder_speed.unwrap()));
            } else {
                new_contents.push_str(line);
                new_contents.push_str("\n");
            }
        }
    }

    new_contents
}


pub fn replace_setrpm_with_socket_send(contents: &String, postprocess: &String)->String{
    let re = if postprocess == "rapid" {
        Regex::new(r"SetRPM(\d+)").unwrap()
    } else {
        Regex::new(r"SetRPM\s(\d+)").unwrap() // Note the space here
    };

    let mut new_contents = String::new();

    for lines in contents.lines(){
        if let Some(captures) = re.captures(lines){
            let number_str = captures.get(1).unwrap().as_str(); //get the number (match group1)
            let number = number_str.parse::<f32>().unwrap(); // get it to a number
            new_contents.push_str(&format!("    SocketSend my_socket \\Str := \"F{}\";\n", number));
        }
        else{
            new_contents.push_str(lines); // Append the original line
            new_contents.push_str("\n"); // Add the newline back
        }
    }
    new_contents
}

pub fn replace_m_code_with_socket_send(contents: &String, postprocess: &String)->String{
    let re = if postprocess == "rapid" {
        Regex::new(r"M_RunCode(\d+)").unwrap()
    } else {
        Regex::new(r"M_RunCode\s(\d+)").unwrap() // Note the space here
    };

    let mut new_contents = String::new();

    for lines in contents.lines(){
        if let Some(captures) = re.captures(lines){
            let number_str = captures.get(1).unwrap().as_str(); //get the number (match group1)
            let number = number_str.parse::<f32>().unwrap(); // get it to a number
            new_contents.push_str(&format!("    SocketSend my_socket \\Str := \"M{}\";\n", number));
        }
        else{
            new_contents.push_str(lines); // Append the original line
            new_contents.push_str("\n"); // Add the newline back
        }
    }
    new_contents
}