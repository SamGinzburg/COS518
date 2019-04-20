use tarpc::{client, context, server};
use tarpc_bincode_transport::{connect, listen, Transport};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::io;

use sharedlib::rpc::new_stub;

pub async fn rpc_get(server_addr: &str, port: u16) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let transport = await!(connect(&server_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    let hello = await!(client.get(context::current(), "Sam".to_string())).unwrap();

    println!("{}", hello);
    
    Ok(())
}
