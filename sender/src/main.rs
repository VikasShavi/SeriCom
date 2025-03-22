use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Read, Write};
use std::path::Path; 
use std::time;
use std::thread;
use serialport::{DataBits, FlowControl, Parity, StopBits};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 5 {
        eprintln!("Usage: {} <port> <baudrate> <sleep_duration> <file_or_directory_paths...>", args[0]);
        return;
    }

    let port_name = &args[1];
    let baud_rate: u32 = args[2].parse().expect("Provide a valid baud rate");
    let sleep_duration: u64 = args[3].parse().expect("Please provide a valid no, sleep time in seconds");

    let mut port = serialport::new(port_name, baud_rate)
        .timeout(time::Duration::from_secs(2))
        .data_bits(DataBits::Eight)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .flow_control(FlowControl::None)
        .open()
        .expect("not able to open serial port");

    println!("*****************************************************");
    println!("Connected to port no: {}", port_name);

    for path_str in &args[4..] {
        let path = Path::new(path_str);

        if path.is_dir() {
            loop {
                println!("*****************************************************");
                for entry in fs::read_dir(path).expect("Not able to directory") {
                    let entry = entry.expect("Not able to read directory");
                    let file_path = entry.path();
                    if file_path.is_file() {
                        if send_file(&file_path, &mut port) {
                            println!("{} sent and deleted!!!", &file_path.file_name()
                                .expect("not able to extract file name").to_str()
                                .expect("name is not valid or contain bad characters"));
                            println!("------------------------------------------------------------");
                            fs::remove_file(&file_path).expect("not able to delete file");
                        }
                    }
                }
                println!("All files sent.");
                println!("Sleeping for {} seconds...", sleep_duration);
                thread::sleep(time::Duration::from_secs(sleep_duration));
            }
        } else if path.is_file() {
            if send_file(path, &mut port) {
                println!("{} sent and deleted!", &path.file_name()
                            .expect("not able to extract file name").to_str()
                            .expect("name is not valid or contain bad characters"));
                println!("------------------------------------------------------------");
                fs::remove_file(path).expect("not able to delete file");
            }
        } else {
            eprintln!("Invalidd path: {}", path_str);
        }
    }
}

fn send_file(file_path: &Path, port: &mut Box<dyn serialport::SerialPort>) -> bool {
    let file_name = file_path
        .file_name()
        .expect("not able to extract file name")
        .to_str()
        .expect("name is not valid or contain bad characters");

    match port.write_all(format!("filename: {}\n", file_name).as_bytes()) {
        Ok(_) => {
            println!("Sent file name: {}", file_name);
            io::stdout().flush().unwrap();
        }
        Err(e) => {
            eprintln!("Error in sending file name: {:?}", e);
            return false;
        }
    }

    let buffer_size = 4096;

    let mut file = File::open(file_path).expect("not able to open file");

    let mut buffer = vec![0; buffer_size];

    loop {
        let bytes_read = file.read(&mut buffer).expect("not able to read from file");
        if bytes_read == 0 {
            break;
        }

        match port.write(&buffer[..bytes_read]) {
            Ok(_) => {
                io::stdout().flush().unwrap();
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => { eprintln!("Error in sending data: {:?}", e); return false }
        }

        thread::sleep(time::Duration::from_millis(200));
    }

    let eof_marker = "UNIQUE_EOF_MARKER_1234";
    match port.write(eof_marker.as_bytes()) {
        Ok(_) => {
            println!("Sent EOF marker for {}", file_name);
            io::stdout().flush().unwrap();
        }
        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
        Err(e) => { eprintln!("Error in sending EOF marker: {:?}", e); return false },
    }
    thread::sleep(time::Duration::from_secs(1));
    return true;
}
