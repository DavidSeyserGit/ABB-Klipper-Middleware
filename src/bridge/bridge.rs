use std::net::{TcpListener, TcpStream, IpAddr};
use std::io::Read;
use reqwest;
use std::error::Error;
use tokio::runtime::Runtime;
mod auth;
use auth as cr;
use std::fs;
use log::{info, warn, error, debug}; // Import logging macros

// --- REMOVE THIS LINE: use log4rs::file::Config as Log4rsFileConfig; ---
// We don't need to import log4rs::file::Config because we're not using it directly
// to deserialize your config.toml. log4rs::init_file handles the log4rs config.


// Load YOUR OWN config at startup (external file)
#[derive(serde::Deserialize)]
struct Config {
    listener_ip: String,
    auth_token: String,
    whitelist: Option<Vec<String>>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // --- Initialize logging with log4rs FIRST ---
    // Load configuration from log4rs.yaml
    log4rs::init_file("log4rs.yaml", Default::default())
        .map_err(|e| format!("Failed to initialize logger: {}", e))?; // Convert error to Box<dyn Error> compatible string
    info!("Application starting...");

    // Ensure the log directory exists (log4rs won't create it for rolling_file)
    if let Err(e) = fs::create_dir_all("log") {
        if e.kind() != std::io::ErrorKind::AlreadyExists {
            eprintln!("Warning: Could not create log directory 'log': {}", e);
        }
    }


    // --- Security check: config.toml must be a file, not a dir ---
    let config_path = "config.toml";

    // --- Now load it ---
    let config_str = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_str)?; // Use YOUR Config struct here

    let auth_token = &config.auth_token;
    info!("Generated Auth Token: {}", auth_token);

    // --- Bind to configured IP ---
    let listener = TcpListener::bind(&config.listener_ip)?;
    info!("✅ Listening on: {}", config.listener_ip);

    // Create a single Tokio runtime instance for the application's lifetime
    let rt = Runtime::new()?;

    // Loop to accept incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let peer_socket_addr = stream.peer_addr().map_err(|e| {
                    error!("Could not get peer address: {:?}", e);
                    e
                })?;

                let peer_ip_addr = match peer_socket_addr.ip() {
                    IpAddr::V4(ip) => ip.to_string(),
                    IpAddr::V6(ip) => ip.to_string(),
                };
                info!("New connection from: {} (socket: {})", peer_ip_addr, peer_socket_addr);

                // --- Single-line Whitelist Check ---
                if config.whitelist.as_ref().map_or(false, |list| !list.contains(&peer_ip_addr)) {
                    warn!("Connection from {} rejected: Not in whitelist.", peer_ip_addr);
                    continue;
                }
                // --- End Whitelist Check ---

                let expected_auth_token = cr::calculate_response(&auth_token);

                // --- Token Validation Logic ---
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(sz) => {
                        if sz == 0 {
                            warn!("Connection from {} closed immediately after connect (no token sent).", peer_ip_addr);
                            continue;
                        }
                        let received_token = String::from_utf8_lossy(&buffer[..sz]).to_string().trim().to_string();

                        if received_token == expected_auth_token {
                            info!("Authentication successful for {} with token.", peer_ip_addr);
                            handle_client(&mut stream, &rt)?;
                        } else {
                            warn!("Authentication failed for {}: Invalid token '{}'. Expected '{}'", peer_ip_addr, received_token, expected_auth_token);
                        }
                    }
                    Err(e) => {
                        error!("Error reading initial token from {}: {:?}", peer_ip_addr, e);
                    }
                }
            }
            Err(err) => {
                error!("Error accepting connection: {:?}", err);
            }
        }
    }

    Ok(())
}

// Function to handle subsequent messages from an authenticated client
fn handle_client(stream: &mut TcpStream, rt: &Runtime) -> Result<(), Box<dyn Error>> {
    let peer_socket_addr = stream.peer_addr().map_err(|e| {
        error!("Could not get peer address for client handler: {:?}", e);
        e
    })?;
    let peer_ip_addr = match peer_socket_addr.ip() {
        IpAddr::V4(ip) => ip.to_string(),
        IpAddr::V6(ip) => ip.to_string(),
    };

    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(sz) => {
                if sz == 0 {
                    info!("Connection from {} closed.", peer_ip_addr);
                    break;
                }
                let received_data = String::from_utf8_lossy(&buffer[..sz]).to_string();
                debug!("Received data from {}: {}", peer_ip_addr, received_data);

                let result = rt.block_on(post_to_moonraker(&received_data));
                match result {
                    Ok(_response) => {
                        debug!("Successfully posted data to Moonraker for {}.", peer_ip_addr);
                    },
                    Err(e) => {
                        error!("Error posting to Moonraker for {}: {:?}", peer_ip_addr, e);
                    }
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    warn!("Read operation would block for {}. Closing connection.", peer_ip_addr);
                } else {
                    error!("Error reading from stream for {}: {:?}", peer_ip_addr, e);
                }
                break;
            }
        }
    }
    Ok(())
}


async fn post_to_moonraker(data: &str) -> Result<reqwest::Response, reqwest::Error> {
    debug!("Preparing G-code: G1 {}", data);
    let fromated_data = format!("G1 {}", data);
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:7125/printer/gcode/script")
        .query(&[("script", fromated_data)])
        .send()
        .await?;
    debug!("Moonraker response: {:?}", response);
    Ok(response)
}