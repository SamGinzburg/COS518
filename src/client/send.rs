use tarpc::{client, context};
use tarpc_bincode_transport::connect;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use sharedlib::rpc::Response::get;

async fn send(server_addr: &str, port: u16) -> () {
	let client_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let transport = await!(connect(&client_addr)).unwrap();

    // new_stub is generated by the service! macro. Like Server, it takes a config and any
    // Transport as input, and returns a Client, also generated by the macro.
    // by the service mcro.
    let mut new_client = await!(client::new(client::Config::default(), transport)).unwrap();

    // The client has an RPC method for each RPC defined in service!. It takes the same args
    // as defined, with the addition of a Context, which is always the first arg. The Context
    // specifies a deadline and trace information which can be helpful in debugging requests.
    let hello = get("Sam".to_string());

    //println!("{}", hello);

	Ok(());
}