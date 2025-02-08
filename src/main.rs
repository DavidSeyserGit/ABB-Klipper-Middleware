use std::net::TcpListener;
use std::io::Read;

fn main()->Result<(), std::io::Error>{
    let listener = TcpListener::bind("127.0.0.1:6969").unwrap();
    for stream in listener.incoming(){
        match stream{
            Ok(mut stream)=>{
                let mut buffer = [0; 1024];
                loop{
                    match stream.read(&mut buffer){
                        Ok(sz)=>{
                            let buffer_string = &buffer[..sz-2];
                            let recieved_data = String::from_utf8_lossy(buffer_string);
                            
                            //instead of printing i should handle the data and send it to the
                            //endpoint
    
                            println!("{:?}", recieved_data);
                        },
                        Err(_) => todo!()

                    };
                }

            }
            Err(err)=>{
                eprintln!("Error: {:?}", err)
            }
        }
    }
    
    Ok(())
}


fn handle_client(){
    todo!()
//we need to implement the json rpc2 connection 
}
