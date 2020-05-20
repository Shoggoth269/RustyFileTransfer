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

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
enum Status {
    Sending(String),
    Receiving,
}

#[derive(Serialize, Deserialize, Debug)]
struct Handshake {
    status: Status,
    filename: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:37626")?;
    let t_pool = ThreadPool::new(8);
    // TODO: Arc<Mutex<Vec<Arc<Mutex<TcpStream>>>>> - Would this allow grabbing a single stream and allowing other threads access to the pool?
    let pending_receivers: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    //let (sender, receiver) = mpsc::channel();

    // println!("{:?}", listener);

    for stream in listener.incoming() {
        let mut stream = stream?;

        let connection_pool = Arc::clone(&pending_receivers);
        t_pool.execute(move || {
            // handle_connection(stream).unwrap();
            match handle_transfer(&mut stream) {
                Ok(handshake) => {
                    match handshake.status {
                        Status::Receiving => {
                            let mut c_pool = connection_pool.lock().unwrap();

                            // TODO: Fix Unwrap
                            c_pool.push(stream.try_clone().unwrap());
                        } // end match Status::Receiving
                        Status::Sending(ip_receiver) => {

                            // TODO: Fix Unwraps
                            // Match IP address with receiver
                            let mut c_pool = connection_pool.lock().unwrap();
                            // TODO: Fix Unwrap
                            let mut receiving_connection = None;
                            // TODO: Vec::Retain can automatically remove the stream when we find it
                            for connection in c_pool.iter_mut() {
                                // Should match fine with the string from sender
                                let ip_address = connection.peer_addr().unwrap().ip().to_string();
                                if ip_receiver == ip_address {
                                    // We have the right connection
                                    // TODO: Can we get a lock on this connection and release the lock on connection_pool?
                                    // TODO: Should get a reference instead of try_clone() so that we can later remove from vector?

                                    println!("ip_address: {}\nip_receiver: {}\nconnection: {:?}", ip_address, ip_receiver, connection);

                                    receiving_connection = match connection.try_clone() {
                                        Ok(s) => Some(s),
                                        Err(_) => panic!("Error when cloning stream from connection pool."),
                                    };
                                    break;
                                }
                            } // end for

                            let mut receiving_connection = match receiving_connection {
                                Some(s) => s,
                                None => panic!("Error: no stream found for given IP address: \n{:?}", c_pool),
                            };

                            // Use the connection to perform the file transfer from stream to connection
                            assert!(!handshake.filename.is_empty());

                            let filename_handshake = Handshake {
                                status: Status::Receiving, // We don't need to use the status this time
                                filename: handshake.filename,
                            };
                            let serialized_handshake = bincode::serialize(&filename_handshake).unwrap();
                            
                            match receiving_connection.write_all(&serialized_handshake) {
                                Err(_) => panic!("Error when sending filename handshake."),
                                Ok(_) => {},
                            }

                            let mut buf = Vec::with_capacity(51200);
                            // Continuously pass bytes from sender to receiver
                            loop {
                                match stream.read(&mut buf) {
                                    Ok(n) => {
                                        // if n is less than contents.len(), we should be done
                                        // if n == 0, we are definitely done
                    
                                        // Write all contents of buffer from 0 to n (number of bytes read)
                                        // TODO: Fix Unwrap
                                        // TODO: Do we need to reset the contents buffer?
                                        receiving_connection.write_all(&buf[0..n]).unwrap();
                    
                                        // Do we also need to check if n is less than 51200 (total size of buffer)?
                                        if n == 0 {
                                            break;
                                        }
                                    },
                                    Err(_) => panic!("Error when reading from sender."),
                                }
                            }

                            // TODO: When done sending file, can we remove stream from vector?

                        } // end match Status::Sending
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

fn handle_transfer(stream: &mut TcpStream) -> Result<Handshake, Box<dyn Error>> {
    let mut handshake_bytes = [0u8 , 2];
    stream.read_exact(&mut handshake_bytes)?;
    let handshake_bytes = (((handshake_bytes[0] as u16) << 8) | (handshake_bytes[1] as u16)) as usize;

    let mut buf = Vec::with_capacity(handshake_bytes);
    stream.read_exact(&mut buf)?;
    let deserialized_handshake: Handshake = bincode::deserialize(&buf)?;

    Ok(deserialized_handshake)
}