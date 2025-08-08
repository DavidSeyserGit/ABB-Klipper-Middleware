use std::sync::{Arc, Mutex};
use warp::Filter;
use serde::Serialize;

use crate::client::ClientInfo;
use crate::dashboard::{STATS, DashboardStats};

#[derive(Serialize)]
struct StatsResponse {
    data_sent: usize,
    data_received: usize,
    last_command: String,
}

#[derive(Serialize)]
struct ClientInfoResponse {
    ip: String,
    connected_at_secs: u64,
    last_activity_secs: u64,
    status: String,
}

pub async fn run_api(clients: Arc<Mutex<std::collections::HashMap<String, ClientInfo>>>) {
    // /stats endpoint
    let stats_route = warp::path("stats").map(|| {
        let stats = STATS.lock().unwrap();
        let resp = StatsResponse {
            data_sent: stats.data_sent,
            data_received: stats.data_received,
            last_command: stats.last_command.clone(),
        };
        warp::reply::json(&resp)
    });

    // /clients endpoint
    let clients_filter = warp::any().map(move || Arc::clone(&clients));
    let clients_route = warp::path("clients")
        .and(clients_filter)
        .map(|clients: Arc<Mutex<std::collections::HashMap<String, ClientInfo>>>| {
            let map = clients.lock().unwrap();
            let mut list = Vec::new();
            for c in map.values() {
                list.push(ClientInfoResponse {
                    ip: c.ip.clone(),
                    connected_at_secs: c.connected_at.elapsed().as_secs(),
                    last_activity_secs: c.last_activity.elapsed().as_secs(),
                    status: c.status.clone(),
                });
            }
            warp::reply::json(&list)
        });

    let routes = stats_route.or(clients_route);

    println!("HTTP API running on http://0.0.0.0:8080");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
