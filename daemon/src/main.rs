#![windows_subsystem = "windows"]

use std::{env, process};
use std::borrow::Cow;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use serde_json::{from_value, json, Value};
use single_instance::SingleInstance;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Notify;
use tray_item::{IconSource, TrayItem};

const DAEMON_IP: &str = "127.0.0.1";
const DAEMON_PORT: u16 = 57222;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    r#type: String,
    data: Value
}

#[derive(Serialize, Deserialize, Debug)]
struct ArgsMessage {
    args: Vec<String>
}

fn get_tcp_address() -> String {
    format!("{}:{}", DAEMON_IP, DAEMON_PORT)
}

async fn handle_client(mut stream: TcpStream) {
    let mut buffer: Vec<u8> = vec![0; 1024];
    let n: usize = stream.read(&mut buffer).await.unwrap();
    let buffer: Cow<str> = String::from_utf8_lossy(&buffer[..n]);

    // Attempt to deserialize the JSON.
    let message: Message = match serde_json::from_str(&buffer) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to deserialize arguments: {}", e);
            return;
        },
    };

    handle_message(message)
}

fn handle_message(message: Message) {
    match message.r#type.as_str() {
        "args" => {
            // Parse the arguments.
            let args_message: ArgsMessage = ArgsMessage { args: from_value(message.data).unwrap() };

            // Pass the arguments along so they can be handled.
            handle_args(args_message.args)
        },
        _ => {
            println!("Unknown message type: {}", message.r#type);
        }
    }
}

fn handle_args(args: Vec<String>) {
    println!("Args: {:?}", args);
}

fn generate_message(r#type: &str, data: Value) -> Value {
    json!({
        "type": r#type,
        "data": data
    })
}

async fn start_server() {
    // Start the TCP server.
    let listener: TcpListener = TcpListener::bind(get_tcp_address()).await.unwrap();

    println!("TCP server started, listening on {}:{}", DAEMON_IP, DAEMON_PORT);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                // Spawn a new task to handle each client.
                tokio::spawn(async move {
                    handle_client(stream).await;
                });
            },
            Err(e) => eprintln!("Failed to accept TCP connection: {}", e),
        }
    }
}

async fn setup_tray(notify: Arc<Notify>) {
    let mut tray_item: TrayItem = TrayItem::new(
        "Frogworks",
        IconSource::Resource("frogworks-logo")
    ).unwrap();

    // Add the tray item's label.
    tray_item.add_label("Frogworks").unwrap();

    // Add the right-click menu item(s) for the tray item.
    tray_item.add_menu_item("Quit", || {
        process::exit(0);
    }).unwrap();

    println!("Setup tray.");

    notify.notified().await
}

async fn send_to_running_instance(message: Value) -> tokio::io::Result<()> {
    // Attempt to connect to the running instance's TCP server.
    let mut stream: TcpStream = TcpStream::connect(get_tcp_address()).await?;

    // Send the message to the daemon.
    stream.write_all(message.to_string().as_bytes()).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let app_name: &str = "frogworks_daemon";
    let instance: SingleInstance = SingleInstance::new(app_name).unwrap();

    // Check if an instance of the daemon is already running.
    if !instance.is_single() {
        eprintln!("There is already a daemon instance running.");

        // Collect the command line arguments.
        let args: Vec<String> = env::args().skip(1).collect();

        // Serialize the arguments into JSON.
        let json_args: Value = json!(args);

        // Generate the message to be sent to the active daemon.
        let message: Value = generate_message("args", json_args);

        // Send the message.
        if let Err(e) = send_to_running_instance(message).await {
            eprintln!("Failed to send message to running instance: {}", e);
        }

        return;
    }

    println!("Starting daemon instance...");

    let notify: Arc<Notify> = Arc::new(Notify::new());
    let notify_clone: Arc<Notify> = notify.clone();

    // Start the TCP server in a separate task.
    tokio::spawn(async move {
        start_server().await;
    });

    // Set up the system tray.
    tokio::spawn(async move {
        setup_tray(notify_clone).await;
    });

    notify.notified().await;
}