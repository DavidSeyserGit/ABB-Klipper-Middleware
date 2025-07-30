use std::net::{TcpListener, TcpStream};
use std::io::Read;
use reqwest;
use std::error::Error;
use tokio::runtime::Runtime;
mod auth;
use auth as cr;
use std::fs;

// Load config at startup (external file)
#[derive(serde::Deserialize)]
struct Config {
    listener_ip: String,
    auth_token: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // --- Security check: config.toml must be a file, not a dir ---
    let config_path = "config.toml";

    // --- Now load it ---
    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?;

    let auth_token = &config.auth_token;
    println!("Generated Auth Token: {}", auth_token);

    // --- Bind to configured IP ---
    let listener = TcpListener::bind(&config.listener_ip)?;
    println!("✅ Listening on: {}", config.listener_ip);

    // Create a single Tokio runtime instance for the application's lifetime
    // This is more efficient than creating one in each loop iteration.
    let rt = Runtime::new()?;

    // Loop to accept incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let peer_addr = stream.peer_addr().map(|addr| addr.to_string()).unwrap_or_else(|_| "unknown".to_string());
                println!("New connection from: {}", peer_addr);
                
                let expected_auth_token = cr::calculate_response(&auth_token);

                // --- Token Validation Logic ---
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(sz) => {
                        if sz == 0 {
                            eprintln!("Connection from {} closed immediately after connect (no token sent).", peer_addr);
                            continue; // Move to the next incoming connection
                        }
                        let received_token = String::from_utf8_lossy(&buffer[..sz]).to_string().trim().to_string(); // Trim whitespace

                        if received_token == expected_auth_token {
                            println!("Authentication successful for {} with token.", peer_addr);
                            // Continue to process subsequent messages in a loop
                            handle_client(&mut stream, &rt)?; // Pass stream and runtime for subsequent messages
                        } else {
                            eprintln!("Authentication failed for {}: Invalid token '{}'. Expected '{}'", peer_addr, received_token, expected_auth_token);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error reading initial token from {}: {:?}", peer_addr, e);
                    }
                }
            }
            Err(err) => {
                eprintln!("Error accepting connection: {:?}", err);
            }
        }
    }

    Ok(())
}

// Function to handle subsequent messages from an authenticated client
fn handle_client(stream: &mut TcpStream, rt: &Runtime) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr().map(|addr| addr.to_string()).unwrap_or_else(|_| "unknown".to_string());
    let mut buffer = [0; 1024]; // Re-use buffer
    loop {
        match stream.read(&mut buffer) {
            Ok(sz) => {
                if sz == 0 {
                    // Connection closed by client
                    println!("Connection from {} closed.", peer_addr);
                    break;
                }
                let received_data = String::from_utf8_lossy(&buffer[..sz]).to_string();
                println!("Received data from {}: {}", peer_addr, received_data);

                let result = rt.block_on(post_to_moonraker(&received_data));
                match result {
                    Ok(_response) => {
                        // Optionally, send a success response back to the client
                        print!("OK");
                    },
                    Err(e) => {
                        eprintln!("Error posting to Moonraker for {}: {:?}", peer_addr, e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading from stream for {}: {:?}", peer_addr, e);
                break;
            }
        }
    }
    Ok(())
}


async fn post_to_moonraker(data: &str) -> Result<reqwest::Response, reqwest::Error> {
    //on the realy robot data will only be a E-Value and an int not a G-Code
    //i have to also accept the F Value so i have to differentiate if the value is E or F type
    //currently expects to get only a int Value -> check if the string contains F or E depending which type it is
    println!("G1 {}", data);
    let fromated_data = format!("G1 {}", data); //format it so that we have a G1 in front of the data
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:7125/printer/gcode/script")
        .query(&[("script", fromated_data)])
        .send()
        .await?;
        println!("{:?}", response);
    Ok(response)
}
