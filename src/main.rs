use std::net::TcpListener;
use std::io::{Read, Write};
use reqwest;
use std::error::Error;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:6969")?;
    println!("Listening on 127.0.0.1:6969");

    let rt = Runtime::new()?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                loop {
                    match stream.read(&mut buffer) {
                        Ok(sz) => {
                            if sz == 0 {
                                // Connection closed
                                break;
                            }
                            let received_data = String::from_utf8_lossy(&buffer[..sz]).to_string();
                            println!("Received data: {}", received_data);

                            let result = rt.block_on(post_to_moonraker(&received_data));
                            match result {
                                Ok(response) => println!("Response: {:?}", response),
                                Err(e) => eprintln!("Error posting to Moonraker: {:?}", e),
                            }
                        }
                        Err(e) => {
                            eprintln!("Error reading from stream: {:?}", e);
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    Ok(())
}

async fn post_to_moonraker(data: &str) -> Result<reqwest::Response, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.post("http://127.0.0.1:7125/printer/gcode/script")
        .query(&[("script", data)])
        .send()
        .await?;
    Ok(response)
}
