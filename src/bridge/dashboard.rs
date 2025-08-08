use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use chrono::Local;
use colored::*;
use crossterm::{cursor, execute, terminal};

use crate::client::ClientInfo;

pub struct DashboardStats {
    pub data_sent: usize,
    pub data_received: usize,
    pub last_command: String,
}

lazy_static::lazy_static! {
    pub static ref STATS: Arc<Mutex<DashboardStats>> = Arc::new(Mutex::new(DashboardStats {
        data_sent: 0,
        data_received: 0,
        last_command: String::new(),
    }));
}

pub fn draw_dashboard(clients: &Arc<Mutex<HashMap<String, ClientInfo>>>) {
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0)
    )
    .unwrap();

    // Header with uptime, system stats, and Moonraker status
    let start_time = Local::now(); // In the real implementation, store globally instead
    let uptime = chrono::Utc::now()
        .signed_duration_since(start_time)
        .to_std()
        .unwrap_or_default();
    let uptime_str = format!("{:02}:{:02}:{:02}",
        uptime.as_secs() / 3600,
        (uptime.as_secs() % 3600) / 60,
        uptime.as_secs() % 60
    ).bright_cyan();

    let active_count = clients.lock().unwrap().len();
    let total_count = active_count; // placeholder — could be cumulative connections
    let cpu_usage = 20.0; // placeholder — integrate sysinfo crate for real values
    let ram_used = 8192; // placeholder — in MB
    let ram_total = 8192; // placeholder
    let moonraker_status = "🔴 Offline".bright_red(); // placeholder — real API check
    // Fetch real-time values from shared stats
    let stats_guard = STATS.lock().unwrap();
    let data_sent = stats_guard.data_sent.to_string().bright_yellow();
    let data_recv = stats_guard.data_received.to_string().bright_yellow();
    let last_command = stats_guard.last_command.clone();
    drop(stats_guard);

    println!(
        "{} @ {}",
        "ABB Klipper Middleware".bright_green().bold(),
        Local::now().format("%Y-%m-%d %H:%M:%S").to_string().bright_white()
    );
    println!(
        "Uptime: {} | Active: {} | Total: {} | CPU: {:.1}% | RAM: {} MB / {} MB | Moonraker: {}",
        uptime_str,
        active_count.to_string().bright_green(),
        total_count.to_string().bright_white(),
        cpu_usage,
        ram_used,
        ram_total,
        moonraker_status
    );
    println!(
        "Data Sent: {} bytes | Data Received: {} bytes | Last Command: {}",
        data_sent,
        data_recv,
        last_command.to_string()
    );

    let term_width = terminal::size().map(|(w, _)| w).unwrap_or(100) as usize;
    println!("{}", "─".repeat(term_width).bright_black());

    // Column titles with dynamic width scaling
    let col_widths = [20, 12, 12, 9, 15]; // IP, Connected, Last Act, % Act, Status
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
        width_stat = col_widths[4] + dynamic_width,
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
