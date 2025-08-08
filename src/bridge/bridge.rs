use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use colored::*;
use crossterm::{cursor, execute, terminal};
use log::{info, error, warn, debug};
use reqwest;
use tokio::runtime::Runtime;

mod auth;
use auth as cr;
use std::error::Error;
use std::fs;

#[derive(serde::Deserialize)]
struct Config {
    listener_ip: String,
    auth_token: String,
    whitelist: Option<Vec<String>>,
}

#[derive(Clone)]
struct ClientInfo {
    ip: String,
    connected_at: Instant,
    last_activity: Instant,
    status: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging from log4rs.yaml
    log4rs::init_file("log4rs.yaml", Default::default())?;

    // Shared state for dashboard
    let clients: Arc<Mutex<HashMap<String, ClientInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Start dashboard thread
    {
        let clients = Arc::clone(&clients);
        thread::spawn(move || loop {
            draw_dashboard(&clients);
            thread::sleep(Duration::from_millis(500));
        });
    }

    // Load config
    let config_str = fs::read_to_string("config.toml")?;
    let config: Config = toml::from_str(&config_str)?;

    println!("{}", "ABB Klipper Middleware".bright_green().bold());
    println!(
        "{} {}",
        "📅 Started at:".bright_yellow(),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
    );
    info!("ABB Klipper Middleware starting...");
    info!("Listening on {}", config.listener_ip);

    let listener = TcpListener::bind(&config.listener_ip)?;

    // ✅ Wrap Runtime in Arc so it can be cloned into threads
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
                if config
                    .whitelist
                    .as_ref()
                    .map_or(false, |list| !list.contains(&peer_ip_addr))
                {
                    warn!("Rejected connection from {}", peer_ip_addr);
                    println!(
                        "{} {}",
                        "⛔ Rejected connection:".bright_red(),
                        peer_ip_addr
                    );
                    continue;
                }

                let expected_auth_token = cr::calculate_response(&config.auth_token);

                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
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
                                    ClientInfo {
                                        ip: peer_ip_addr.clone(),
                                        connected_at: Instant::now(),
                                        last_activity: Instant::now(),
                                        status: "✅ Authenticated".to_string(),
                                    },
                                );
                            }
                            println!(
                                "{} {}",
                                "✅ Authenticated:".bright_green(),
                                peer_ip_addr
                            );
                            info!("Authentication successful for {}", peer_ip_addr);

                            let clients_clone = Arc::clone(&clients);
                            let rt_clone = Arc::clone(&rt); // ✅ Clone Arc<Runtime>
                            let mut stream_clone = stream.try_clone()?;
                            let ip_clone = peer_ip_addr.clone();

                            thread::spawn(move || {
                                handle_client(
                                    &mut stream_clone,
                                    &rt_clone,
                                    &ip_clone,
                                    clients_clone,
                                )
                                .unwrap_or_else(|e| {
                                    error!("Client {} error: {:?}", ip_clone, e)
                                });
                            });
                        } else {
                            warn!(
                                "Invalid token from {} (got '{}', expected '{}')",
                                peer_ip_addr, received_token, expected_auth_token
                            );
                            println!(
                                "{} {}",
                                "❌ Authentication failed for:".bright_red(),
                                peer_ip_addr
                            );
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

fn handle_client(
    stream: &mut TcpStream,
    rt: &Arc<Runtime>, // ✅ Now takes Arc<Runtime>
    ip: &str,
    clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(sz) => {
                if sz == 0 {
                    let mut map = clients.lock().unwrap();
                    if let Some(c) = map.get_mut(ip) {
                        c.status = "🔌 Disconnected".to_string();
                    }
                    println!(
                        "{} {}",
                        "🔌 Connection closed:".bright_yellow(),
                        ip
                    );
                    info!("Connection closed for {}", ip);
                    break;
                }
                let received_data = String::from_utf8_lossy(&buffer[..sz]).to_string();

                {
                    let mut map = clients.lock().unwrap();
                    if let Some(c) = map.get_mut(ip) {
                        c.last_activity = Instant::now();
                        c.status = format!("📡 Sent: {}", received_data);
                    }
                }

                debug!("{} → Sending to Moonraker: {}", ip, received_data);
                let _ = rt.block_on(post_to_moonraker(&received_data));
            }
            Err(e) => {
                let mut map = clients.lock().unwrap();
                if let Some(c) = map.get_mut(ip) {
                    c.status = "❌ Error".to_string();
                }
                error!("Error reading from {}: {:?}", ip, e);
                break;
            }
        }
    }
    Ok(())
}

async fn post_to_moonraker(
    data: &str,
) -> Result<reqwest::Response, reqwest::Error> {
    let formatted_data = format!("G1 {}", data);
    let client = reqwest::Client::new();
    let response = client
        .post("http://127.0.0.1:7125/printer/gcode/script")
        .query(&[("script", formatted_data)])
        .send()
        .await?;
    Ok(response)
}

fn draw_dashboard(clients: &Arc<Mutex<HashMap<String, ClientInfo>>>) {
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    // Header
    println!(
        "{} @ {}",
        "Active Connection Stats".bright_magenta().bold(),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string().bright_white()
    );
    println!("{}", "─".repeat(100).bright_black());

    // Column titles with dynamic width scaling
    let term_width = terminal::size().map(|(w, _)| w).unwrap_or(100) as usize;
    let col_widths = [20, 12, 12, 9, 15, 7]; // Default widths
    let total_static: usize = col_widths.iter().sum::<usize>() + col_widths.len() - 1;
    let dynamic_width = if term_width > total_static {
        term_width - total_static
    } else {
        0
    };

    println!(
        "{:<width_ip$} {:<width_conn$} {:<width_last$} {:<width_pct$} {:<width_stat$}",
        "IP".bright_white().bold(),
        "Connected".bright_white().bold(),
        "Last Act".bright_white().bold(),
        "% Act".bright_white().bold(),
        "Status".bright_white().bold(),
        width_ip = col_widths[0],
        width_conn = col_widths[1],
        width_last = col_widths[2],
        width_pct = col_widths[3],
        width_stat = col_widths[4] + dynamic_width, // give extra space to Status dynamically
    );
    println!("{}", "─".repeat(term_width).bright_black());

    let map = clients.lock().unwrap();
    for client in map.values() {
        let seconds_since_activity = client.last_activity.elapsed().as_secs();
        let percent_active = if seconds_since_activity < 5 {
            100
        } else if seconds_since_activity < 30 {
            80
        } else if seconds_since_activity < 60 {
            50
        } else {
            10
        };

        let percent_colored = if percent_active >= 80 {
            format!("{}%", percent_active).bright_green()
        } else if percent_active >= 50 {
            format!("{}%", percent_active).bright_yellow()
        } else {
            format!("{}%", percent_active).bright_red()
        };

        println!(
            "{:<width_ip$} {:<width_conn$} {:<width_last$} {:<width_pct$} {:<width_stat$}",
            client.ip.bright_cyan(),
            format!("{:?}", client.connected_at.elapsed()).bright_green(),
            format!("{:?}", client.last_activity.elapsed()).bright_yellow(),
            percent_colored,
            client.status.clone(),

            width_ip = col_widths[0],
            width_conn = col_widths[1],
            width_last = col_widths[2],
            width_pct = col_widths[3],
            width_stat = col_widths[4] + dynamic_width,
        );
    }
    println!("{}", "─".repeat(term_width).bright_black());
}
