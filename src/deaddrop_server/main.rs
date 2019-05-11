#![feature(futures_api, arbitrary_self_types, await_macro, async_await)]

#[macro_use] extern crate lazy_static;

extern crate clap;
extern crate rand;
extern crate sharedlib;
extern crate tarpc;
extern crate tarpc_bincode_transport;
extern crate tokio;

use crate::tarpc::futures::{compat::Executor01CompatExt, FutureExt, TryFutureExt};

use tarpc::server::Handler;
use tarpc_bincode_transport::listen;

use sharedlib::deaddrop_rpc::serve;
use sharedlib::deaddrop_rpc::DeadDropServer;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tarpc::server;

use clap::{App, Arg};
use std::collections::HashMap;

lazy_static! {
    // quick hack to get args into callback function without modifying the
    // cursive lib / making a custom UI object
    static ref HASHMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        let matches = App::new("Vuvuzela Deaddrop Server")
                        .version("1.0")
                        .about("Vuvuzela Deaddrop Server")
                        .author("Sam Ginzburg")
                        .author("Benjamin Kuykendall")
                        .arg(Arg::with_name("addr")
                            .short("a")
                            .long("addr")
                            .help("Specifies the IPv4 addr of the head server in the Vuvuzela chain")
                            .takes_value(true))
                        .arg(Arg::with_name("port")
                            .short("p")
                            .long("port")
                            .help("Specifies the port of the head server in the Vuvuzela chain")
                            .takes_value(true))
                        .arg(Arg::with_name("micro")
                            .short("m")
                            .long("micro")
                            .help("Specifies the value of μ, for differential privacy")
                            .takes_value(true))
                        .get_matches();

        let server_ip = String::from(matches.value_of("addr").unwrap_or("127.0.0.1").clone());
        let server_port = String::from(matches.value_of("port").unwrap_or("8082").clone());
        let micro = String::from(matches.value_of("micro").unwrap_or("10").clone());
        
        m.insert(String::from("micro"), micro);
        m.insert(String::from("server_ip"), server_ip);
        m.insert(String::from("server_port"), server_port);
        m.clone()
    };
}


async fn run_service(server_addr: &str, port: u16) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = listen(&server_addr)?;
    let _addr = transport.local_addr();

    let micro: f64 = match HASHMAP.get(&String::from("micro")) {
        // param was passed
        Some(x) => x.parse::<f64>().unwrap(),
        // no param!
        None => panic!("No input provided for the micro flag!"),
    };

    // The server is configured with the defaults.
    let server = server::new(server::Config::default())
        // Server can listen on any type that implements the Transport trait.
        .incoming(transport)
        // serve is generated by the service! macro. It takes as input any type implementing
        // the generated Service trait.
        .respond_with(serve(DeadDropServer{ micro: micro }));

    await!(server);

    Ok(())
}

fn main() {
    tarpc::init(tokio::executor::DefaultExecutor::current().compat());

    let ip = HASHMAP.get(&String::from("server_ip")).unwrap();
    let port = HASHMAP
        .get(&String::from("server_port"))
        .unwrap()
        .parse::<u16>()
        .unwrap();

    tokio::run(
        run_service(ip, port)
            .map_err(|e| eprintln!("RPC Error: {}", e))
            .boxed()
            .compat(),
    );
}
