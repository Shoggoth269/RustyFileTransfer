use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::error::Error;
use std::fs;
use std::thread;
use std::time::Duration;
use thread_pool::ThreadPool;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

enum Status {
    Sending,
    Receiving,
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:37625")?;
    let t_pool = ThreadPool::new(8);
    let pending_receivers: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    //let (sender, receiver) = mpsc::channel();

    // println!("{:?}", listener);

    for stream in listener.incoming() {
        let mut stream = stream?;

        let connection_pool = Arc::clone(&pending_receivers);
        t_pool.execute(move || {
            // handle_connection(stream).unwrap();
            match handle_transfer(&mut stream) {
                Ok(status) => {
                    match status {
                        Status::Receiving => {
                            let mut c_pool = connection_pool.lock().unwrap();
                            c_pool.push(stream);
                        }
                        Status::Sending => {
                            // Get IP address from sender
                            

                            // Match IP address with receiver
                            let mut c_pool = connection_pool.lock().unwrap();
                            for connection in c_pool.iter_mut() {
                                // Should match fine with the string from sender
                                let ip_address = connection.peer_addr().unwrap().ip().to_string();
                            }
                        }
                    }
                }
                Err(_) => return,
            }
        });
    }

    println!("Shutting down.");

    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>>  {
    let mut buffer = [0; 512];
    stream.read(&mut buffer)?;

    // TODO: Consider logging
    // println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK\r\n\r\n", "hello.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND\r\n\r\n", "404.html")
    };

    let contents = fs::read_to_string(filename)?;

    let response = format!("{}{}", status_line, contents);

    stream.write(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

fn handle_transfer(stream: &mut TcpStream) -> Result<Status, Box<dyn Error>> {
    let mut buffer = [0; 512];
    stream.read(&mut buffer)?;

    let sending = b"SENDING";
    let receiving = b"RECEIVING";

    if buffer.starts_with(sending) {
        return Ok(Status::Sending);
    } else if buffer.starts_with(receiving) {
        return Ok(Status::Receiving);
    }

    Err("Invalid command".into())
}