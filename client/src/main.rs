use std::io::prelude::*;
use std::net::{Shutdown, TcpStream};
use std::{io, str, thread, time};
use std::io::{ErrorKind, stdin};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8088")?;

    let read = stream.try_clone().unwrap();
    let write = stream.try_clone().unwrap();

    let username = get_username();

    let rcv_thread_handle = thread::spawn(move || {
        rcv_msg_handler(&read).unwrap();
    });

    let send_thread_handle = thread::spawn(move || {
        send_msg_handler(username, &write).unwrap();
    });

    send_thread_handle.join();
    rcv_thread_handle.join();
    Ok(())
}

fn rcv_msg_handler(mut stream:  &TcpStream) -> std::io::Result<()>{
    loop{
        let mut rcv_data = [0; 2000];

        match stream.read(&mut rcv_data) {
            Ok(msg_size) => {
                print!("{}", str::from_utf8(&rcv_data[0..msg_size]).unwrap());
            }
            Err(ref e) if e.kind() == ErrorKind::TimedOut => {
                drop(stream);
                thread::sleep(Duration::from_millis(100));
                continue;
            }
            Err(e) => {
                eprintln!("{}", e);
                stream.shutdown(Shutdown::Both).unwrap();
                return Err(e);
            }
        }
    }

}

fn send_msg_handler(username : String, mut stream:  &TcpStream)-> std::io::Result<()>{

    loop{
        let mut input_buffer = String::from(&username);
        let line_size = stdin().read_line(&mut input_buffer).unwrap();

        match stream.write(input_buffer[0..input_buffer.len()].as_bytes()){
            Ok(stream) => {}
            Err(e) => {
                eprintln!("{}", e);
                stream.shutdown(Shutdown::Both).unwrap();
                return Err(e);
            }
        }
    }
}

fn get_username() -> String{
    let mut username_buffer = String::new();
    println!("Type your username to chat: ");
    let name_len = stdin().read_line(&mut username_buffer).unwrap();
    username_buffer = username_buffer.trim().to_string();
    username_buffer.push_str(": ");
    username_buffer
}