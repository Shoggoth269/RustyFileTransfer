use std::collections::HashMap;
// use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::{BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::Path;
use std::str;
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Start by collecting our public IP address
    // let resp = reqwest::blocking::get("https://api.ipify.org?format=json")?
    //     .json::<HashMap<String, String>>()?;
    // println!("{:#?}", resp);

    let response = reqwest::blocking::get("https://api.ipify.org?format=json");
    let public_ip = match response {
        Ok(res) => match res.json::<HashMap<String, String>>() {
            Ok(res_map) => res_map,
            Err(_) => panic!("Error when assigning JSON result for public IP address."),
        },
        Err(_) => panic!("Error when attempting to retrieve public IP address."),
    };

    // Throw away the hashmap and keep the IP address only
    println!("{:#?}", public_ip);
    let public_ip = match public_ip.get("ip") {
        Some(ip) => ip,
        None => panic!("Error when unpacking IP address from JSON (hashmap)."),
    };
    println!("Your public IP Address: {:#?}\n", public_ip);

    // Determine whether we are sending or receiving
    let mut reader = BufReader::new(io::stdin());
    let mut input: Vec<u8> = Vec::new();
    let mut input_str: String;

    loop {
        print!("Select an option:\n\n\t1. Send a file\n\t2. Receive a file\n:> ");
        // TODO: Fix Expect
        io::stdout().flush().expect("flush failed!");
        // TODO: Fix Unwrap
        reader.read_until(b'\n', &mut input).unwrap();
        input_str = String::from_utf8_lossy(&input).into_owned();

        input.clear();
        

        if input_str.ends_with('\n') {
            input_str.pop();
        }
        if input_str.ends_with('\r') {
            input_str.pop();
        }

        if input_str != "1" && input_str != "2" {
            println!("Error: enter a valid option.");
        } else {
            println!("");
            break;
        }
    }

    // Connect to the server
    let server_ip = String::from("104.237.129.224:37626");
    let mut stream = match TcpStream::connect(&server_ip) {
        Ok(stream) => stream,
        Err(_) => panic!("Error when connecting to server IP: {:#?}", server_ip),
    };

    // C:\Users\Shogg\Desktop\Game Soundtrack Downloader (Python 3).py
    // Send a file
    if input_str == "1" {
        
        print!("Enter remote IP. E.g.: 127.0.0.1\n:> ");
        // TODO: Fix Expect
        io::stdout().flush().expect("flush failed!");

        reader.read_until(b'\n', &mut input).unwrap();
        let mut remote_ip = String::from_utf8_lossy(&input).into_owned();

        input.clear();

        if remote_ip.ends_with('\n') {
            remote_ip.pop();
        }
        if remote_ip.ends_with('\r') {
            remote_ip.pop();
        }

        // TODO: Send info to server and wait for confirmation before going further
        // TODO: Pack info into a struct and serialize/deserialize rather than dumb protocol ("Filename::")

        print!("Enter file name to send.\n:> ");
        // TODO: Fix Expect
        io::stdout().flush().expect("flush failed!");

        reader.read_until(b'\n', &mut input).unwrap();
        let mut filename = String::from_utf8_lossy(&input).into_owned();

        input.clear();

        if filename.ends_with('\n') {
            filename.pop();
        }
        if filename.ends_with('\r') {
            filename.pop();
        }

        // Open file
        let path = Path::new(&filename);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(error) => panic!(
                "Error when creating file {}: {}",
                display,
                error.to_string()
            ),
            Ok(file) => file,
        };
        
        let handshake = Handshake {
            status: Status::Sending(remote_ip),
            filename: match path.file_name() {
                Some(name) => match name.to_str() {
                    Some(name) => String::from(name),
                    None => panic!("Error when getting file name from path (internal)."),
                },
                None => panic!("Error when getting file name from path."),
            },
        };
        // TODO: Fix Unwrap
        let serialized_handshake = bincode::serialize(&handshake).unwrap();

        // Write size of serialized_handshake
        let handshake_bytes = [(serialized_handshake.len() >> 8) as u8, 
                                        (serialized_handshake.len() & 0xFF) as u8];

        match stream.write_all(&handshake_bytes) {
            Err(_) => panic!("Error when sending handshake bytes."),
            Ok(_) => {},
        }

        match stream.write_all(&serialized_handshake) {
            Err(_) => panic!("Error when sending handshake."),
            Ok(_) => {}
        }

        // Create a buffer with size of 50 KB
        let mut contents = Vec::with_capacity(51200);
        loop {
            match file.read(&mut contents) {
                Ok(n) => {
                    // if n is less than contents.len(), we should be done
                    // if n == 0, we are definitely done

                    // Write all contents of buffer from 0 to n (number of bytes read)
                    // TODO: Fix Unwrap
                    // TODO: Do we need to reset the contents buffer?
                    stream.write_all(&contents[0..n]).unwrap();

                    // Do we also need to check if n is less than 51200 (total size of buffer)?
                    if n == 0 {
                        break;
                    }
                },
                Err(_) => panic!("Error when reading from file."),
            }
        }
    } // End file send
    // Receive a file
    else {
        // Start by creating a Handshake to let the server know we are receiving
        // Filename is useless
        // TODO: change filename to Option?
        let handshake = Handshake {
            status: Status::Receiving,
            filename: String::from(""),
        };

        // TODO: Fix Unwrap
        let serialized_handshake = bincode::serialize(&handshake).unwrap();

        // Write size of serialized_handshake
        let handshake_bytes = [(serialized_handshake.len() >> 8) as u8, 
                                        (serialized_handshake.len() & 0xFF) as u8];

        match stream.write_all(&handshake_bytes) {
            Err(_) => panic!("Error when sending handshake bytes: {:?}, {:?}", handshake_bytes, serialized_handshake),
            Ok(_) => {},
        }

        match stream.write_all(&serialized_handshake) {
            Err(_) => panic!("Error when sending handshake."),
            Ok(_) => {}
        }

        println!("Provide your public IP address to the sender to begin receiving file.");

        // Receive Handshake from server
        // TODO: Fix Unwraps
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf).unwrap();
        let deserialized_handshake: Handshake = bincode::deserialize(&buf).unwrap();

        let filename = deserialized_handshake.filename;
        assert!(!filename.is_empty());

        let path = Path::new(&filename);
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("Error when creating file {}: {}", display, why.to_string()),
            Ok(file) => file,
        };

        // Create a buffer with size of 50 KB
        let mut buf = Vec::with_capacity(51200);
        loop {
            
            // TODO: Fix Unwrap
            match stream.read(&mut buf) {
                Ok(n) => {
                    match file.write_all(&buf) {
                        Err(_) => panic!("Error when writing bytes to file."),
                        Ok(_) => {}
                    }

                    if n == 0 {
                        break;
                    }
                }
                Err(_) => panic!("Error when reading from stream."),
            }            
        }
    } // End file receive

    Ok(())
}
