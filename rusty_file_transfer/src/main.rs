use std::collections::HashMap;
use std::io::{stdin, stdout, Read, Write, SeekFrom};
use text_io::read;
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::error::Error;

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
    let mut input: char;

    loop {
        print!("Select an option:\n\n\t1. Send a file\n\t2. Receive a file\n:>");
        input = read!();
        if input != '1' && input != '2' {
            println!("Error: enter a valid option.");
        }
        else {
            println!("");
            break;
        }
    }
    
    // C:\Users\Shogg\Desktop\Game Soundtrack Downloader (Python 3).py
    // Send a file
    if input == '1' {
        println!("Enter remote IP and port. E.g.: 127.0.0.1:80\n:>");
        let remote_ip: String = read!("{}\n");
        let mut remote_ip: String = read!("{}\n");

        if remote_ip.ends_with("\r")
        {
            remote_ip.pop();
        }

        let mut stream = match TcpStream::connect(&remote_ip) {
            Ok(stream) => stream,
            Err(_) => panic!("Error when connecting to remote IP: {:#?}", remote_ip),
        };

        println!("Enter file name to send.\n:>");
        let mut filename: String = read!("{}\n");
        if filename.ends_with("\r")
        {
            filename.truncate(filename.len() - 1);
        }
        // Open file
        let path = Path::new(&filename);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(error) => panic!("Error when creating file {}: {}", display, error.to_string()),
            Ok(file) => file,
        };
        
        let filename_send: String = String::from("Filename::") + match path.file_name() {
            Some(name) => match name.to_str() {
                Some(name) => name,
                None => panic!("Error when getting file name from path (internal)."),
            },
            None => panic!("Error when getting file name from path."),
        };
        let size: [u8; 2] = [(filename_send.as_bytes().len() as u16 >> 8) as u8, (filename_send.as_bytes().len() as u16 & 0xFF) as u8];

        // Debug Prints
        // println!("\n\nNum bytes: {}\n\n", ((size[0] as u16) << /*fix dumb highlighting>*/ 8 | (size[1] as u16)) as usize);
        // println!("\n\nSize Array: {:#?}", size);
        // println!("\n\nNum bytes (original): {}\n\n", filename_send.as_bytes().len());

        match stream.write_all(&size) {
            Err(_) => panic!("Error when sending size."),
            Ok(_) => {},
        }

        match stream.write_all(filename_send.as_bytes()) {
            Err(_) => panic!("Error when sending file name."),
            Ok(_) => {},
        }

        let file_size = match file.seek(SeekFrom::End(0)) {
            Ok(file_size) => file_size,
            Err(_) => panic!("Error when getting length of file."),
        };
        // Go back to the start of the file
        match file.seek(SeekFrom::Start(0)) {
            Ok(_) => {},
            Err(_) => panic!("Error when moving to beginning of file."),
        }

        loop {
            // check how far from end, read 1024 or until end, send
            let current_position = match file.seek(SeekFrom::Current(0)) {
                Ok(current_position) => current_position,
                Err(_) => panic!("Error when getting current position of file."),
            };

            if (file_size - current_position) >= 1024 {
                // Go ahead and do a full 1024 bytes
                let mut bytes = [0; 1024];
                match file.read_exact(&mut bytes) {
                    Ok(_) => {},
                    Err(_) => panic!("Error when reading bytes from file."),
                }

                let size: [u8; 2] = [(bytes.len() as u16 >> 8) as u8, (bytes.len() as u16 & 0xFF) as u8];

                match stream.write_all(&size) {
                    Err(_) => panic!("Error when sending size."),
                    Ok(_) => {},
                }

                match stream.write_all(&mut bytes) {
                    Ok(_) => {},
                    Err(_) => panic!("Error when writing bytes to stream."),
                }
            }
            else {
                let mut bytes: Vec<u8> = Vec::new();
                match file.read_to_end(&mut bytes) {
                    Ok(_) => {},
                    Err(_) => panic!("Error when reading bytes from file."),
                }

                // Signify the end of the file
                bytes.push('E' as u8);
                bytes.push('O' as u8);
                bytes.push('F' as u8);
                bytes.push('F' as u8);
                bytes.push('O' as u8);
                bytes.push('E' as u8);

                let size: [u8; 2] = [(bytes.len() as u16 >> 8) as u8, (bytes.len() as u16 & 0xFF) as u8];

                match stream.write_all(&size) {
                    Err(_) => panic!("Error when sending size."),
                    Ok(_) => {},
                }

                match stream.write_all(&mut bytes) {
                    Ok(_) => {},
                    Err(_) => panic!("Error when writing bytes to stream."),
                }

                break;
            }
        }

    } // End file send
    // Receive a file
    else {
        let listener = match TcpListener::bind("127.0.0.1:0") {
            Ok(listen) => listen,
            Err(_) => panic!("Error when creating TCPListener."),
        };

        println!("Listener created with port: {:?}\nProvide your public IP address and port to the sender.", match listener.local_addr() {
            Ok(port) => port.port(),
            Err(_) => panic!("Error when getting port from TCPListener."),
        });

        // Once we get a connection, we first receive the filename and extension
        // From there, will will continue receiving bytes until we reach a terminator sequence
        let mut stream = match listener.accept() {
            Ok((stream, _)) => stream,
            Err(_) => panic!("Error when accpeting incoming connection."),
        };

        let mut size: [u8; 2] = [0; 2];

        // Read filename and create file
        // Read two bytes that tell us how many bytes will be in the next chunk
        // This is used because read_exact requires knowing exactly how many bytes will be received
        // Without this knowledge, it can lead to invalid data in our buffer
        match stream.read_exact(&mut size) {
            Err(_) => panic!("Error while receiving size bytes."),
            Ok(_) => {}, // do nothing
        }
        
        let mut bytes: Vec<u8> = Vec::new();
        bytes.resize(((size[0] as u16) << /*fix dumb highlighting>*/ 8 | (size[1] as u16)) as usize, 0); // Inner comment is to fix bad syntax highlighting due to requiring matching <>
        match stream.read_exact(&mut bytes) {
            Err(_) => panic!("Error while receiving data bytes."),
            Ok(_) => {}, // do nothing
        }

        // Debug Print
        // println!("\n\nNum bytes: {}\n\n", ((size[0] as u16) << /*fix dumb highlighting>*/ 8 | (size[1] as u16)) as usize);

        // File format is "Filename::file.txt"
        // If filename is empty, we populate the filename
        let filename: String = match str::from_utf8(&bytes) {
            Ok(name) => name.to_string(),
            Err(_) => panic!("Error when parsing string for filename."),
        };

        // Open file
        // let tokens: Vec<&str> = filename.split("::").collect();
        let tokens: Vec<String> = filename.split("::").collect::<Vec<&str>>().into_iter().map(|x| x.to_string()).collect();
        if tokens.len() != 2 {
            panic!("Error when splitting file name: {}", filename);
        }
        let path = Path::new(&tokens[1]);
        let display = path.display();

        let mut file = match File::create(&path) {
            Err(why) => panic!("Error when creating file {}: {}", display, why.to_string()),
            Ok(file) => file,
        };

        loop {
            // Read two bytes that tell us how many bytes will be in the next chunk
            // This is used because read_exact requires knowing exactly how many bytes will be received
            // Without this knowledge, it can lead to invalid data in our buffer
            match stream.read_exact(&mut size) {
                Err(_) => panic!("Error while receiving size bytes."),
                Ok(_) => {}, // do nothing
            }
            
            let mut bytes: Vec<u8> = Vec::new();
            bytes.resize(((size[0] as u16) << /*fix dumb highlighting>*/ 8 | (size[1] as u16)) as usize, 0); // Inner comment is to fix bad syntax highlighting due to requiring matching <>
            match stream.read_exact(&mut bytes) {
                Err(_) => panic!("Error while receiving data bytes."),
                Ok(_) => {}, // do nothing
            }

            // Check last six bytes of vector, if we always add these bytes to our last chunk, we know it will be long enough to avoid errors (in the case that we sent less than 6 useful bytes)
            let mut end = String::new();
            for index in bytes.len() - 6 .. bytes.len() {
                end.push(bytes[index] as char);
            }
            if end == "EOFFOE" {
                // Throw away the last six bytes and end after this
                bytes.truncate(bytes.len() - 6);
                match file.write_all(&bytes) {
                    Err(_) => panic!("Error when writing bytes to file."),
                    Ok(_) => {},
                }
                break;
            }
            else {
                // Save all bytes and continue to the next chunk
                match file.write_all(&bytes) {
                    Err(_) => panic!("Error when writing bytes to file."),
                    Ok(_) => {},
                }
            }
        }
    } // End file receive


    Ok(())
}
