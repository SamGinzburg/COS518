use std::{thread, time, io};
use tarpc::{client, context, server};
use tarpc_bincode_transport::{connect, listen, Transport};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use sharedlib::head_rpc::new_stub;
use crate::futures::TryFutureExt;
use crate::futures::FutureExt;

pub async fn rpc_get(server_addr: String, port: u16) -> io::Result<()> {
	// keep fetching data on a regular interval
	let one_sec = time::Duration::from_millis(1000);
	let socket_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
	let transport = await!(connect(&socket_addr)).unwrap();
	let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
	let response = await!(client.get(context::current(), 5, 5)).unwrap();
	// TODO: log response somewhere
	//println!("{}", response);
	thread::sleep(one_sec);
	Ok(())
}