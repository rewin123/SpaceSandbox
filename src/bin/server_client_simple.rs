
use std::net::*;

fn main() {

    let server_addr = "127.0.0.1:1996";
    let client_addr = "0.0.0.0:0";

    let server_socket = UdpSocket::bind(server_addr).unwrap();
    let client_socket = UdpSocket::bind(client_addr).unwrap();

    client_socket.connect(server_addr).unwrap();


    let test_data = "Hello my custom UPD";
    client_socket.send(test_data.as_bytes()).unwrap();

    let mut buf = [0u8; 512];
    let recv_size = server_socket.recv(&mut buf).unwrap();

    println!("Recv size: {}", recv_size);

    println!("Test text: {}", String::from_utf8(buf[0..recv_size].to_vec()).unwrap());
}