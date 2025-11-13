use std::net::TcpListener;
use std::io::Read;
use reqwest;
use std::error::Error;
use tokio::runtime::Runtime;

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:6969")?; // needs to be changed to the robot IP or 0.0.0.0:6969 10.0.0.10:1234
    let rt = Runtime::new()?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let _ = stream.peer_addr().map(|addr| println!("Connection successful with: {}", addr));
                let mut buffer = [0; 1024];
                loop {
                    match stream.read(&mut buffer) {
                        Ok(sz) => {
                            if sz == 0 {
                                // Connection closed
                                break;
                            }
                            let received_data = String::from_utf8_lossy(&buffer[..sz]).to_string();

                            let result = rt.block_on(post_to_moonraker(&received_data));
                            match result {
                                Ok(_response) => (),
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
