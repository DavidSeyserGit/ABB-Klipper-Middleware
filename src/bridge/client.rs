use std::collections::HashMap;
use std::io::Read;
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use std::error::Error;
use log::{info, error, debug};
use tokio::runtime::Runtime;
use crate::dashboard::STATS;
use crate::dashboard::DashboardStats;

#[derive(Clone)]
pub struct ClientInfo {
    pub ip: String,
    pub connected_at: Instant,
    pub last_activity: Instant,
    pub status: String,
}

impl ClientInfo {
    pub fn new_authenticated(ip: String) -> Self {
        Self {
            ip,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            status: "✅ Connected".to_string(),
        }
    }
}

pub fn handle_client(
    stream: &mut TcpStream,
    rt: &Arc<Runtime>,
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
                    println!("🔌 Connection closed: {}", ip);
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

                // Update stats for data_received and last_command
                {
                    let mut stats = STATS.lock().unwrap();
                    stats.data_received += sz;
                    stats.last_command = received_data.clone();
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
    let res_body = data.as_bytes().len();
    {
        let mut stats = STATS.lock().unwrap();
        stats.data_sent += res_body;
    }
    let response = client
        .post("http://127.0.0.1:7125/printer/gcode/script")
        .query(&[("script", formatted_data)])
        .send()
        .await?;
    Ok(response)
}
