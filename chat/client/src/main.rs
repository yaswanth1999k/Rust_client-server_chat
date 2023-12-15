use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::{SystemTime, Duration};

const LOCAL: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;

fn main() {
    println!("Enter your username:");
    let mut username = String::new();
    io::stdin().read_line(&mut username).expect("Failed to read username");
    let username = username.trim().to_string();  // Clone the username into a String

    let mut client = TcpStream::connect(LOCAL).expect("Stream failed to connect");
    client.set_nonblocking(true).expect("Failed to initiate non-blocking");

    // Send username to the server
    client.write_all(username.as_bytes()).expect("Failed to send username");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let username = username.clone();  // Clone the username into the closure
        loop {
            let mut buff = vec![0; MSG_SIZE];
            match client.read_exact(&mut buff) {
                Ok(_) => {
                    let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                    let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                    println!("message recv {:?}", msg);
                },
                Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                Err(_) => {
                    println!("connection with server was severed");
                    break;
                }
            }

            match rx.try_recv() {
                Ok(msg) => {
                    let timestamp = SystemTime::now();
                    let timestamp_str = match timestamp.duration_since(SystemTime::UNIX_EPOCH) {
                        Ok(duration) => duration.as_secs().to_string(),
                        Err(_) => "0".to_string(),
                    };
                    let msg_with_timestamp = format!("{} {}: {}", username, timestamp_str, msg);

                    let mut buff = msg_with_timestamp.into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).expect("writing to socket failed");
                    println!("message sent {:?}", msg);
                },
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => break,
            }

            thread::sleep(Duration::from_millis(100));
        }
    });

    println!("Write a Message:");
    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("reading from stdin failed");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() { break }
    }
    println!("bye bye!");
}
