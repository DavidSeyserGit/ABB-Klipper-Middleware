mod auth;
mod dashboard;
mod client;
mod config;

use std::net::TcpListener;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use log::{info, warn, error};
use tokio::runtime::Runtime;
use chrono::Local;
use colored::*;

use crate::client::handle_client;
use crate::auth as cr;
use crate::dashboard::{draw_dashboard, STATS};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    log4rs::init_file("log4rs.yaml", Default::default())?;

    // Shared state for dashboard
    let clients = Arc::new(Mutex::new(std::collections::HashMap::new()));

    // Start dashboard thread
    {
        let clients = Arc::clone(&clients);
        thread::spawn(move || loop {
            draw_dashboard(&clients);
            thread::sleep(Duration::from_millis(500));
        });
    }

    // Load config
    let config = config::load_config("config.toml")?;

    println!("{}", "ABB Klipper Middleware".bright_green().bold());
    println!(
        "{} {}",
        "📅 Started at:".bright_yellow(),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    );
    info!("ABB Klipper Middleware starting...");
    info!("Listening on {}", config.listener_ip);

    let listener = TcpListener::bind(&config.listener_ip)?;
    let rt = Arc::new(Runtime::new()?);

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let peer_socket_addr = stream.peer_addr()?;
                let peer_ip_addr = peer_socket_addr.ip().to_string();
                println!(
                    "{} {} ({})",
                    "🔌 New connection from:".bright_blue(),
                    peer_ip_addr.bright_white(),
                    peer_socket_addr
                );
                info!("New connection from {} ({})", peer_ip_addr, peer_socket_addr);

                // Whitelist check
                if config.whitelist.as_ref().map_or(false, |list| !list.contains(&peer_ip_addr)) {
                    warn!("Rejected connection from {}", peer_ip_addr);
                    println!("{} {}", "⛔ Rejected connection:".bright_red(), peer_ip_addr);
                    continue;
                }

                let expected_auth_token = cr::calculate_response(&config.auth_token);

                let mut buffer = [0; 1024];
                use std::io::Read;
                match Read::read(&mut stream, &mut buffer) {
                    Ok(sz) => {
                        if sz == 0 {
                            warn!("Empty connection from {}", peer_ip_addr);
                            continue;
                        }
                        let received_token = String::from_utf8_lossy(&buffer[..sz])
                            .trim()
                            .to_string();

                        if received_token == expected_auth_token {
                            {
                                let mut map = clients.lock().unwrap();
                                map.insert(
                                    peer_ip_addr.clone(),
                                    crate::client::ClientInfo::new_authenticated(peer_ip_addr.clone()),
                                );
                            }
                            println!("{} {}", "✅ Authenticated:".bright_green(), peer_ip_addr);
                            info!("Authentication successful for {}", peer_ip_addr);

                            let clients_clone = Arc::clone(&clients);
                            let rt_clone = Arc::clone(&rt);
                            let mut stream_clone = stream.try_clone()?;
                            let ip_clone = peer_ip_addr.clone();

                            thread::spawn(move || {
                                if let Err(e) = handle_client(&mut stream_clone, &rt_clone, &ip_clone, clients_clone) {
                                    error!("Client {} error: {:?}", ip_clone, e);
                                }
                            });
                        } else {
                            warn!(
                                "Invalid token from {} (got '{}', expected '{}')",
                                peer_ip_addr, received_token, expected_auth_token
                            );
                            println!("{} {}", "❌ Authentication failed for:".bright_red(), peer_ip_addr);
                        }
                    }
                    Err(e) => {
                        error!("Error reading token from {}: {:?}", peer_ip_addr, e);
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
