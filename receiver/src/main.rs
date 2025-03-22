use mio::{Events,Interest, Poll, Token};
use std::env;
use std::fs::OpenOptions;
use std::io;
use std::io::{Read, Write};
use mio_serial::SerialPortBuilderExt;
use chrono::Local;

const SERIAL_TOKEN: Token = Token(0);
const BUFFER_SIZE: usize = 4096;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!("Usage: {} <port> <baudrate>", args[0]);
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "not enough arguments"));
    }

    let path = &args[1];
    let baud_rate: u32 = args[2].parse().expect("Please provide a valid baud rate");

    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1);

    println!("*****************************************************");
    println!("Opening serial path at {} at {},8N1", path, baud_rate);
    println!("*****************************************************");
    let mut rx = mio_serial::new(path, baud_rate).open_native_async()?;

    poll.registry()
        .register(&mut rx, SERIAL_TOKEN, Interest::READABLE)
        .unwrap();

    let mut buf = vec![0u8; BUFFER_SIZE];
    let eof_marker = b"UNIQUE_EOF_MARKER_1234";
    let filename_prefix = b"filename: ";

    loop {
        let mut data_buffer = Vec::new();
        let mut file_name = String::new();
        let mut file_created = false;
        let mut file = None;
        let mut receiving_file = false;

        loop {
            poll.poll(&mut events, None)?;

            for event in events.iter() {
                match event.token() {
                    SERIAL_TOKEN => loop {
                        match rx.read(&mut buf) {
                            Ok(count) => {
                                if count > 0 {
                                    data_buffer.extend_from_slice(&buf[..count]);

                                    if !receiving_file {
                                        if let Some(pos) = data_buffer.windows(filename_prefix.len()).position(|window| window == filename_prefix) {
                                            if let Some(end_pos) = data_buffer[pos + filename_prefix.len()..].iter().position(|&c| c == b'\n') {
                                                file_name = String::from_utf8_lossy(
                                                    &data_buffer[pos + filename_prefix.len()..pos + filename_prefix.len() + end_pos]
                                                ).to_string();
                                                
                                                data_buffer.drain(..pos + filename_prefix.len() + end_pos + 1);

                                                receiving_file = true;
                                                file_created = true;

                                                let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
                                                let parts: Vec<&str> = file_name.split('.').collect();
                                                let extension = if parts.len() > 1 {
                                                    parts[1..].join(".")
                                                } else {
                                                    "dat".to_string()
                                                };
                                                file_name = format!("{}_{}.{}", file_name.trim_end_matches(format!(".{}", extension).as_str()), timestamp, extension);

                                                file = Some(OpenOptions::new()
                                                    .write(true)
                                                    .create(true)
                                                    .truncate(true)
                                                    .open(&file_name)?);

                                                println!("filename received: {}", file_name);
                                            }
                                        }
                                    }

                                    if receiving_file {
                                        if let Some(pos) = data_buffer.windows(eof_marker.len()).position(|window| window == eof_marker) {
                                            if let Some(ref mut file) = file {
                                                file.write_all(&data_buffer[..pos]).unwrap();
                                                file.flush().unwrap();
                                            }
                                            println!("EOF detected, file will be saved as {}. Waiting to receive the next file...", file_name);
                                            println!("------------------------------------------------------------");

                                            receiving_file = false;
                                            file_created = false;
                                            data_buffer.clear();
                                            break;
                                        }

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
                                println!("Quitting due to error while reading the data: {}", e);
                                return Err(e);
                            }
                        }
                    },
                    _ => {}
                }
            }

            if file_created && data_buffer.windows(eof_marker.len()).any(|window| window == eof_marker) {
                break;
            }
        }
    }
}