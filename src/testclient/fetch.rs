use sharedlib::head_rpc::new_stub;
use std::io;
use std::net::{IpAddr, SocketAddr};
use tarpc::client;
use tarpc_bincode_transport::connect;

pub async fn rpc_get(server_addr: String, port: u16) -> io::Result<()> {
    // keep fetching data on a regular interval
    let socket_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&socket_addr)).unwrap();
    let _client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // TODO: write code to fetch message
    // let response = await!(client.get(context::current(), 5, 5)).unwrap();
    // TODO: log response somewhere
    //println!("{}", response);
    Ok(())
}
