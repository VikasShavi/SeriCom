use std::{env, fs::{self, File}, io::{self, Read, Write}, path::Path, time};
use std::thread;
use serialport::{self, DataBits, FlowControl, Parity, StopBits};

fn main() {
    // Collect all command-line arguments into a vector
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} <port> <baudrate> <sleep_duration> <file_or_directory_paths...>", args[0]);
        return;
    }

    let port_name = &args[1];
    let baud_rate: u32 = args[2].parse().expect("Please provide a valid baud rate");
    let sleep_duration: u64 = args[3].parse().expect("Please provide a valid sleep duration");

    // Open the serial port
    let mut port = serialport::new(port_name, baud_rate)
        .timeout(time::Duration::from_secs(2))
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .open()
        .expect("Failed to open serial port");

    println!("*****************************************************");
    println!("Connected to {}", port_name);
    // Iterate over all provided file or directory paths
    for path_str in &args[4..] {
        let path = Path::new(path_str);

        if path.is_dir() {           // If the path is a directory, send all files in the directory
            loop {
                println!("*****************************************************");
                for entry in fs::read_dir(path).expect("Failed to read directory") {
                    let entry = entry.expect("Failed to read directory entry");
                    let file_path = entry.path();
                    if file_path.is_file() {
                        if send_file(&file_path, &mut port) {
                            println!("{} sent and deleted!", &file_path.file_name()
                                .expect("Failed to extract file name").to_str()
                                .expect("File name is not valid UTF-8"));
                            println!("------------------------------------------------------------");
                            fs::remove_file(&file_path).expect("Failed to delete file after sending");
                        }
                    }
                }
                println!("All files sent.");
                println!("Sleeping for {} seconds...", sleep_duration);
                thread::sleep(time::Duration::from_secs(sleep_duration));
            }
        } else if path.is_file() {
            // If the path is a file, send the file
            if send_file(path, &mut port) {
                println!("{} sent and deleted!", &path.file_name()
                            .expect("Failed to extract file name").to_str()
                            .expect("File name is not valid UTF-8"));
                println!("------------------------------------------------------------");
                fs::remove_file(path).expect("Failed to delete file after sending");
            }
        } else {
            eprintln!("Invalid path: {}", path_str);
        }
    }
}

fn send_file(file_path: &Path, port: &mut Box<dyn serialport::SerialPort>) -> bool {
    // Extract the file name from the file path
    let file_name = file_path
        .file_name()
        .expect("Failed to extract file name")
        .to_str()
        .expect("File name is not valid UTF-8");

    // Send the file name followed by a newline character
    match port.write_all(format!("filename: {}\n", file_name).as_bytes()) {
        Ok(_) => {
            println!("Sent file name: {}", file_name);
            io::stdout().flush().unwrap();
        }
        Err(e) => {
            eprintln!("Error sending file name: {:?}", e);
            return false;
        }
    }

    // Buffer size
    let buffer_size = 4096;

    // Open the file
    let mut file = File::open(file_path).expect("Failed to open file");

    // Buffer to hold the data chunks
    let mut buffer = vec![0; buffer_size];

    // Read and send data in chunks
    loop {
        let bytes_read = file.read(&mut buffer).expect("Failed to read from file");
        if bytes_read == 0 {
            break; // End of file
        }

        match port.write(&buffer[..bytes_read]) {
            Ok(_) => {
                // println!("Sent {} bytes", bytes_read);
                io::stdout().flush().unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => { eprintln!("Error sending data: {:?}", e); return false }
        }

        // Wait for 0.2 seconds before sending the next chunk
        thread::sleep(time::Duration::from_millis(200));
    }

    // Send the end-of-file marker
    let eof_marker = "UNIQUE_EOF_MARKER_1234";
    match port.write(eof_marker.as_bytes()) {
        Ok(_) => {
            println!("Sent EOF marker for {}", file_name);
            io::stdout().flush().unwrap();
        }
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => { eprintln!("Error sending EOF marker: {:?}", e); return false },
    }
    // port.flush().unwrap();
    // Add a small delay before sending the next file name
    thread::sleep(time::Duration::from_secs(1));
    return true;
}
