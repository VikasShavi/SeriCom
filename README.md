# **SeriCom**  
## **Introduction**
- Rust-based sender and receiver for efficient file transfers using serial ports. 
Handles errors such as 
  - when the sender transmits data before the receiver starts, 
  - or when the serial port is already in use. 
  
- Designed for continuous operation, the sender and receiver can run indefinitely, allowing users to simply copy files into a designated directory for transfer. 

- Ideal for secure environments where internet access is restricted/closed to protect systems.
  
## **Features**
   - Rust-based implementation.
   - Supports sending and receiving any type of file.
   - Cross-platform compatibility.
  
## Installation

- Rquires `Rust` for building both the sender and receiver components. 
- `serialport` crate for the sender, 
- `mio-serial` crate for the receiver, 
- `docker` and `cross-rs` for cross-compiling.

### Prerequisites
- **Rust**: Install Rust via `rustup` by running:
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
  After installation, check:
  ```bash
  rustc --version
  cargo --version
  ```

> [!note]
> **docker** and **cross-rs** are required only for **cross-compiling**.

- **Docker**: Cross-compiling with `cross-rs` requires Docker to be installed and running. Install Docker from [Docker's official site](https://docs.docker.com/get-docker/).

  Check:
  ```bash
  docker --version
  ```

- **cross-rs**: for cross compiling
  ```bash
  cargo install cross --git https://github.com/cross-rs/cross
  ```

### Sender & Receiver binaries

#### **Compiling for the current host**
  ```bash
  # This will generate an executable with the name sender/sender.exe
  # in ./target/release folder based on the current OS.
  git clone https://github.com/VikasShavi/SeriCom.git
  cd SeriCom/sender
  cargo build --release
  ```

  ```bash
  # This will generate an executable with the name receiver/receiver.exe
  # in ./target/release folder based on the current OS.
  cd SeriCom/receiver
  cargo build --release
  ```

#### **Cross-Compiling**
  > [!warning]
  > Wont work if docker daemon is not running

  - **For windows(x86-64)**
    ```bash
    cross build --target x86_64-pc-windows-gnu
    ```

  - **For linux(x86-64 dynamically linked)**
    ```bash
    cross build --target x86_64-unknown-linux-gnu
    ```

  - **For linux(x86-64 statically linked)**
    ```bash
    cross build --target x86_64-unknown-linux-musl
    ```

## **Usage**
> [!note]
> - Always start the receiver first
> - Get the port name from
>   - **Windows**: `Device Manager --> Ports(COM & LPT)`
>   - **Linux**: `ls /dev/tty.*`


1. **Windows**

    _Receiver_
   	```lua
   	.\receiver.exe <PORT> <BAUD_RATE>
   	
   	-- Example
   	.\receiver.exe COM3 230400
   	```

    _Sender_
   	```lua
   	.\sender.exe <PORT> <BAUD_RATE> <SLEEP_TIME_IN_SEC> <FOLDER_LOCATION:ALL_FILES_FROM_A_FOLDER_TO_SEND>
   	
   	-- Example
   	.\sender.exe COM5 230400 5 c:\windows\temp
   	```

2. **Linux**

    _Receiver_
   	```lua
   	./receiver <PORT> <BAUD_RATE>
   	
   	-- Example
   	./receiver /dev/tty.usbserial-110 230400
   	```

    _Sender_
   	```lua
   	./sender <PORT> <BAUD_RATE> <SLEEP_TIME_IN_SEC> <FOLDER_LOCATION:ALL_FILES_FROM_A_FOLDER_TO_SEND>
   	
   	-- Example
   	./sender /dev/tty.usbserial-130 230400 5 /tmp
   	```

4. **Example**

![](https://drive.google.com/file/d/1zjkLHDSOKQe8rDcYyXQ_ISBpCT33jNVG/view?usp=sharing)

### Technical Details

- **Serial Communication Libraries**: 
  - **Sending**: The project uses the `serialport` crate for sending data over the serial connection.
  - **Receiving**: For receiving data, the `mio-serial` crate is used instead of `serialport` due to baud rate mismatch between the two ends of the serial cable.

- **Reason for Choosing `mio-serial` for Receiving**
    During testing, I encountered a speed issue when attempting to use `serialport` for both sending and receiving. The two ends of the serial cable supported different baud rates, causing asynchronous communication problems when using `serialport` for receiving. The `mio-serial` crate, which offers better handling of asynchronous IO, was chosen to address this mismatch and ensure stable communication across devices with varying baud rate support.

## **Troubleshooting**
   - **Files are not received by the receiver**: This can happen if any of the serial ports are ==Transmit-Only or Receive-Only Devices==. So just interchange the ports for the tools, then run and check once again.

## How to Contribute

Thank you for considering contributing to this project! Contributions, whether they be bug fixes, feature improvements, or new ideas, are welcome.
 
- **Fork the repository**: Click the "Fork" button at the top right of this repository to create a personal copy.
- **Clone the repository**: Clone your fork locally with:
  ```bash
  git clone https://github.com/VikasShavi/SeriCom.git
  ```
- **Create a new branch**: Always work on a new branch to keep your changes isolated.
  ```bash
  git checkout -b feature-or-bugfix-name
  ```
- **Commit your changes**: Keep your commits small and clear. Write descriptive commit messages:
  ```bash
  git commit -m "Add feature/fix bug for..."
  ```
- **Push your branch**:
  ```bash
  git push origin feature-or-bugfix-name
  ```
- **Create a pull request (PR)**: Open a pull request from your branch to the main repository. Add a clear description of what changes were made and why.

## **License**
- MIT License

## **Contact**
  - Email: vikasvivek2000@gmail.com
