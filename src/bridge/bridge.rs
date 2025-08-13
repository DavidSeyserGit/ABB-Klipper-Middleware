use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use chrono::{Local, DateTime};
use colored::*;
use crossterm::{cursor, execute, terminal};
use log::{info, error, warn, debug};
use reqwest;
use tokio::runtime::Runtime;
use sysinfo::{System};

use std::error::Error;
use std::fs;

#[derive(serde::Deserialize)]
struct Config {
    listener_ip: String,
    whitelist: Option<Vec<String>>,
}

#[derive(Clone)]
struct ClientInfo {
    ip: String,
    connected_at: Instant,
    last_activity: Instant,
    status: String,
}

#[derive(Default)]
struct Stats {
    start_time: DateTime<Local>,
    total_connections: usize,
    bytes_sent: usize,
    bytes_received: usize,
    last_command: String,
    moonraker_online: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging from log4rs.yaml
    log4rs::init_file("log4rs.yaml", Default::default())?;

    // Shared state for dashboard
    let clients: Arc<Mutex<HashMap<String, ClientInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let stats = Arc::new(Mutex::new(Stats {
        start_time: Local::now(),
        ..Default::default()
    }));

    // Start dashboard thread
    {
        let clients = Arc::clone(&clients);
        let stats = Arc::clone(&stats);
        thread::spawn(move || loop {
            draw_dashboard(&clients, &stats);
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
            Ok(stream) => {
                let peer_socket_addr = stream.peer_addr()?;
                let peer_ip_addr = peer_socket_addr.ip().to_string();

                println!(
                    "{} {} ({})",
                    "🔌 New connection from:".bright_blue(),
                    peer_ip_addr.bright_white(),
                    peer_socket_addr
                );
                info!("New connection from {} ({})", peer_ip_addr, peer_socket_addr);

                {
                    let mut s = stats.lock().unwrap();
                    s.total_connections += 1;
                }

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

                // Accept connection immediately (auth removed)
                {
                    let mut map = clients.lock().unwrap();
                    map.insert(
                        peer_ip_addr.clone(),
                        ClientInfo {
                            ip: peer_ip_addr.clone(),
                            connected_at: Instant::now(),
                            last_activity: Instant::now(),
                            status: "✅ Connected".to_string(),
                        },
                    );
                }
                println!("{} {}", "✅ Connection accepted:".bright_green(), peer_ip_addr);
                info!("Connection accepted for {}", peer_ip_addr);

                let clients_clone = Arc::clone(&clients);
                let stats_clone = Arc::clone(&stats);
                let rt_clone = Arc::clone(&rt);
                let mut stream_clone = stream.try_clone()?;
                let ip_clone = peer_ip_addr.clone();

                thread::spawn(move || {
                    handle_client(
                        &mut stream_clone,
                        &rt_clone,
                        &ip_clone,
                        clients_clone,
                        stats_clone,
                    )
                    .unwrap_or_else(|e| error!("Client {} error: {:?}", ip_clone, e));
                });
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
    rt: &Arc<Runtime>,
    ip: &str,
    clients: Arc<Mutex<HashMap<String, ClientInfo>>>,
    stats: Arc<Mutex<Stats>>,
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
                {
                    let mut s = stats.lock().unwrap();
                    s.bytes_received += sz;
                }
                let received_data = String::from_utf8_lossy(&buffer[..sz]).to_string();

                {
                    let mut map = clients.lock().unwrap();
                    if let Some(c) = map.get_mut(ip) {
                        c.last_activity = Instant::now();
                        c.status = format!("📡 Sent: {}", received_data);
                    }
                }

                {
                    let mut s = stats.lock().unwrap();
                    s.last_command = received_data.clone();
                }

                debug!("{} → Sending to Moonraker: {}", ip, received_data);
                let res = rt.block_on(post_to_moonraker(&received_data));
                if res.is_ok() {
                    let mut s = stats.lock().unwrap();
                    s.bytes_sent += received_data.len();
                    s.moonraker_online = true;
                } else {
                    let mut s = stats.lock().unwrap();
                    s.moonraker_online = false;
                }
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

fn draw_dashboard(
    clients: &Arc<Mutex<HashMap<String, ClientInfo>>>,
    stats: &Arc<Mutex<Stats>>,
) {
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    let sys = System::new_all();
    let cpu_usage = sys.global_cpu_usage();
    let total_mem = sys.total_memory() / 1024 / 1024;
    let used_mem = (sys.total_memory() - sys.available_memory()) / 1024 / 1024;

    let s = stats.lock().unwrap();
    let uptime = Local::now() - s.start_time;
    let active_connections = clients.lock().unwrap().len();

    println!(
        "{} @ {}",
        "ABB Klipper Middleware".bright_green().bold(),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string().bright_white()
    );
    println!(
        "Uptime: {} | Active: {} | Total: {} | CPU: {:.1}% | RAM: {} MB / {} MB | Moonraker: {}",
        format!("{:02}:{:02}:{:02}", uptime.num_hours(), uptime.num_minutes() % 60, uptime.num_seconds() % 60).bright_cyan(),
        active_connections.to_string().bright_green(),
        s.total_connections.to_string().bright_white(),
        cpu_usage,
        used_mem,
        total_mem,
        if s.moonraker_online { "🟢 Online".bright_green() } else { "🔴 Offline".bright_red() }
    );
    println!(
        "Data Sent: {} bytes | Data Received: {} bytes | Last Command: {}",
        s.bytes_sent.to_string().bright_yellow(),
        s.bytes_received.to_string().bright_yellow(),
        s.last_command.bright_white()
    );
    println!("{}", "─".repeat(100).bright_black());

    println!(
        "{:<20} {:<20} {:<20} {:<30}",
        "IP".bright_white().bold(),
        "Connected".bright_white().bold(),
        "Last Activity".bright_white().bold(),
        "Status".bright_white().bold()
    );
    println!("{}", "─".repeat(100).bright_black());

    let map = clients.lock().unwrap();
    for client in map.values() {
        println!(
            "{:<20} {:<20} {:<20} {:<30}",
            client.ip.bright_cyan(),
            format!("{:?}", client.connected_at.elapsed()).bright_green(),
            format!("{:?}", client.last_activity.elapsed()).bright_yellow(),
            client.status.clone()
        );
    }
    println!("{}", "─".repeat(100).bright_black());
}