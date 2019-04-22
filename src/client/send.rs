use tarpc::{client, context, server};
use tarpc_bincode_transport::{connect, listen, Transport};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::io;
use futures::Stream;
use sharedlib::rpc::new_stub;

pub async fn rpc_put(server_addr: String, port: u16, message: String) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&server_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    let response = await!(client.put(context::current(), message.to_string())).unwrap();

    //println!("{}", response);

    Ok(())
}
