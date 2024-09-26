use mio::{Events, Interest, Poll, Token};
use std::env;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use mio_serial::SerialPortBuilderExt;
use chrono::Local;

const SERIAL_TOKEN: Token = Token(0);
const BUFFER_SIZE: usize = 4096;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <port> <baudrate>", args[0]);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Insufficient arguments"));
    }

    let path = &args[1];
    let baud_rate: u32 = args[2].parse().expect("Please provide a valid baud rate");

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1);

    println!("*****************************************************");
    println!("Opening {} at {},8N1", path, baud_rate);
    println!("*****************************************************");
    let mut rx = mio_serial::new(path, baud_rate).open_native_async()?;

    poll.registry()
        .register(&mut rx, SERIAL_TOKEN, Interest::READABLE)
        .unwrap();

    let mut buf = vec![0u8; BUFFER_SIZE];
    let eof_marker = b"UNIQUE_EOF_MARKER_1234";
    let filename_prefix = b"filename: ";

    loop {
        // Variables for each file transfer
        let mut data_buffer = Vec::new();
        let mut file_name = String::new();
        let mut file_created = false;
        let mut file = None;
        let mut receiving_file = false;  // Added flag to only start when a file name is received

        loop {
            poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    SERIAL_TOKEN => loop {
                        match rx.read(&mut buf) {
                            Ok(count) => {
                                if count > 0 {
                                    data_buffer.extend_from_slice(&buf[..count]);

                                    // Check for the filename prefix if not receiving any file yet
                                    if !receiving_file {
                                        if let Some(pos) = data_buffer.windows(filename_prefix.len()).position(|window| window == filename_prefix) {
                                            // Now check if we have the full file name after the prefix
                                            if let Some(end_pos) = data_buffer[pos + filename_prefix.len()..].iter().position(|&c| c == b'\n') {
                                                file_name = String::from_utf8_lossy(
                                                    &data_buffer[pos + filename_prefix.len()..pos + filename_prefix.len() + end_pos]
                                                ).to_string();
                                                
                                                // Remove everything up to and including the file name from the buffer
                                                data_buffer.drain(..pos + filename_prefix.len() + end_pos + 1);

                                                receiving_file = true;  // Now we are receiving a file
                                                file_created = true;

                                                // Generate a unique file name with timestamp
                                                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                                                let parts: Vec<&str> = file_name.split('.').collect();
                                                let extension = if parts.len() > 1 {
                                                    parts[1..].join(".")
                                                } else {
                                                    "dat".to_string()
                                                };
                                                file_name = format!("{}_{}.{}", file_name.trim_end_matches(format!(".{}", extension).as_str()), timestamp, extension);

                                                // Open the file for writing
                                                file = Some(OpenOptions::new()
                                                    .write(true)
                                                    .create(true)
                                                    .truncate(true)
                                                    .open(&file_name)?);

                                                println!("Filename received: {}", file_name);
                                            }
                                        }
                                    }

                                    // If we are receiving a file, check for the EOF marker
                                    if receiving_file {
                                        // Check for EOF marker in the accumulated data
                                        if let Some(pos) = data_buffer.windows(eof_marker.len()).position(|window| window == eof_marker) {
                                            if let Some(ref mut file) = file {
                                                // Write all data before EOF marker to the file
                                                file.write_all(&data_buffer[..pos]).unwrap();
                                                file.flush().unwrap();
                                            }
                                            println!("EOF detected. File saved as {}. Waiting for the next file...", file_name);
                                            println!("------------------------------------------------------------");

                                            // Reset to prepare for the next file
                                            receiving_file = false;
                                            file_created = false;
                                            data_buffer.clear();
                                            break; // Break the inner loop and reset for the next file
                                        }

                                        // If the buffer reaches BUFFER_SIZE, write it to the file
                                        if data_buffer.len() >= BUFFER_SIZE {
                                            if let Some(ref mut file) = file {
                                                file.write_all(&data_buffer[..BUFFER_SIZE]).unwrap();
                                                file.flush().unwrap();
                                            }
                                            data_buffer.drain(..BUFFER_SIZE);
                                        }
                                    }
                                }
                            }
                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                break;
                            }
                            Err(e) => {
                                println!("Quitting due to read error: {}", e);
                                return Err(e);
                            }
                        }
                    },
                    _ => {}
                }
            }

            // If EOF marker was detected, break the outer loop to reset for the next file
            if file_created && data_buffer.windows(eof_marker.len()).any(|window| window == eof_marker) {
                break;
            }
        }
    }
}