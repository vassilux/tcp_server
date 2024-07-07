use std::io::{self};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:9000").await?;
    let clients: Arc<Mutex<Vec<OwnedWriteHalf>>> = Arc::new(Mutex::new(Vec::new()));

    println!("TCP server listening on 127.0.0.1:9000");

    // Task for reading user input from the terminal
    let clients_clone = Arc::clone(&clients);
    tokio::spawn(async move {
        loop {
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read from stdin");

            let mut clients = clients_clone.lock().await;
            for client in clients.iter_mut() {
                if let Err(e) = client.write_all(input.as_bytes()).await {
                    eprintln!("Failed to write to client: {:?}", e);
                }
            }
        }
    });

    // Accept connections and process them
    loop {
        let (socket, _) = listener.accept().await?;
        let clients_clone = Arc::clone(&clients);
        tokio::spawn(async move {
            let (reader, writer) = socket.into_split();
            let mut reader = BufReader::new(reader);
            clients_clone.lock().await.push(writer);

            loop {
                let mut line = String::new();
                match reader.read_line(&mut line).await {
                    Ok(0) => break, // Connection closed
                    Ok(_) => {
                        println!("Received from client: {}", line);
                    }
                    Err(e) => {
                        eprintln!("Failed to read from socket: {:?}", e);
                        break;
                    }
                }
            }
        });
    }
}
