use std::io::prelude::*;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::{str, thread};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender};

fn main() {
    println!("Starting TCP Server...");

    let listener = create_socket();

    let client_connections = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) : (Sender<String>, Receiver<String>) = mpsc::channel(); //Creating a channel to share data between threads

    let mut client_connections_clone  = Arc::clone(&client_connections);
    let broadcast_handler = thread::spawn(move || { broadcast_msg(rx, &*client_connections_clone ) });

    //Accepting connections and spawning a thread to handle the client, incoming() will infinitely loop
    for stream in listener.incoming(){
        match stream {
            Ok(stream) => {
                let cloned_write_stream = stream.try_clone().unwrap();
                let cloned_tx = tx.clone();

                let mut clients_clone = Arc::clone(&client_connections);
                let mut client_connections_lock = clients_clone.lock().unwrap();
                client_connections_lock.push(cloned_write_stream);
                drop(client_connections_lock); //Free our lock

                thread::spawn(move|| client_handler(stream, cloned_tx));
            }
            Err(err) => { println!("Error {}", err); }
        }
    }

    broadcast_handler.join();
    drop(listener);
}

fn create_socket() -> TcpListener{
    println!("Creating server socket...");
    let listener = TcpListener::bind("127.0.0.1:8088").expect("Failed to bind");
    println!("Server listening...");
    listener
}

fn client_handler(mut stream : TcpStream, tx : Sender<String>){
    stream.write("Welcome to the Rust chat server!".as_bytes());

    'client_loop: loop {
        //Our receiving buffer
        let mut rcv_msg : [u8; 2000] = [0; 2000];

        match stream.read(&mut rcv_msg) { //Listening to our client
            Ok(msg_size) => {
                print!("Server Received: {}", str::from_utf8(&rcv_msg[0..msg_size]).unwrap() );
                let received_msg = str::from_utf8(&rcv_msg[0..msg_size]).unwrap();
                tx.send(String::from(received_msg)).unwrap() //Sharing data across threads
            }
            Err(err) => {
                println!("Terminating connection with {}", stream.peer_addr().unwrap());
                stream.shutdown(Shutdown::Both); //Closing read and write streams
                break;
            }
        }
    }
}

fn broadcast_msg(rx : Receiver<String>, clients : &Mutex<Vec<TcpStream>>) {
    loop {
        match rx.recv(){
            Ok(message) => {
                let mut clients_lock = clients.lock().unwrap();
                clients_lock.iter().for_each(|mut client| { client.write(message.as_bytes()); });
            }
            Err(err) => {
                println!("Error broadcasting message");
                continue;
            }
        }
    }
}
