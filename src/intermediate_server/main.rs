#![feature(futures_api, arbitrary_self_types, await_macro, async_await)]

#[macro_use]
extern crate lazy_static;
extern crate tarpc;

extern crate clap;
extern crate rand;
extern crate sharedlib;
extern crate tarpc_bincode_transport;
extern crate tokio;

use crate::tarpc::futures::compat::Executor01CompatExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::TryFutureExt;
use std::collections::HashMap;

use clap::{App, Arg};

use tarpc::server::Handler;
use tarpc_bincode_transport::listen;

use sharedlib::int_rpc::serve;
use sharedlib::int_rpc::IntermediateServer;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::thread;
use tarpc::server;

lazy_static! {
    // quick hack to get args into callback function without modifying the
    // cursive lib / making a custom UI object
    static ref HASHMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        let matches = App::new("Vuvuzela Intermediate Server")
                        .version("1.0")
                        .about("Vuvuzela Intermediate Server")
                        .author("Sam Ginzburg")
                        .author("Benjamin Kuykendall")
                        // this effectively corresponds to the id # for key lookup
                        .arg(Arg::with_name("server_id")
                            .short("id")
                            .long("server_id")
                            .help("Specifies which server keypair to use")
                            .takes_value(true))
                        // TODO: implement this to allow for n>3 servers in vuvuzela chain
                        .arg(Arg::with_name("forward")
                            .short("f")
                            .long("forward")
                            .help("Flag to tell if we are forwarding to deaddrop or no")
                            .takes_value(true))
                        .arg(Arg::with_name("addr")
                            .short("a")
                            .long("addr")
                            .help("Specifies which addr to bind the RPC server to")
                            .takes_value(true))
                        .arg(Arg::with_name("port")
                            .short("p")
                            .long("port")
                            .help("Specifies which port to bind the RPC server to")
                            .takes_value(true))
                        .arg(Arg::with_name("nextaddr")
                            .long("nextaddr")
                            .help("Specifies the addr of the next server in the chain")
                            .takes_value(true))
                        .arg(Arg::with_name("nextport")
                            .long("nextport")
                            .help("Specifies the port of the next server in the chain")
                            .takes_value(true))
                        .arg(Arg::with_name("prevaddr")
                            .long("prevaddr")
                            .help("Specifies the addr of the previous server in the chain")
                            .takes_value(true))
                        .arg(Arg::with_name("prevport")
                            .long("prevport")
                            .help("Specifies the port of the previous server in the chain")
                            .takes_value(true))
                        .arg(Arg::with_name("micro")
                            .short("m")
                            .long("micro")
                            .help("Specifies the value of μ, for differential privacy")
                            .takes_value(true))
                        .arg(Arg::with_name("variance")
                            .short("b")
                            .long("variance")
                            .help("Specifies the variance of the noise distribution, for differential privacy")
                            .takes_value(true))
                        .get_matches();

        let server_uid = String::from(matches.value_of("server_id").unwrap_or("1").clone());
        let server_ip = String::from(matches.value_of("addr").unwrap_or("127.0.0.1").clone());
        let server_port = String::from(matches.value_of("port").unwrap_or("8081").clone());
        let next_server_ip = String::from(matches.value_of("nextaddr").unwrap_or("127.0.0.1").clone());
        let next_server_port = String::from(matches.value_of("nextport").unwrap_or("8082").clone());
        let prev_server_ip = String::from(matches.value_of("prevaddr").unwrap_or("127.0.0.1").clone());
        let prev_server_port = String::from(matches.value_of("prevport").unwrap_or("8080").clone());
        let micro = String::from(matches.value_of("micro").unwrap_or("10").clone());
        let b = String::from(matches.value_of("variance").unwrap_or("0").clone());

        m.insert(String::from("variance"), b);
        m.insert(String::from("micro"), micro);
        m.insert(String::from("server_id"), server_uid);
        m.insert(String::from("server_ip"), server_ip);
        m.insert(String::from("server_port"), server_port);
        m.insert(String::from("next_server_ip"), next_server_ip);
        m.insert(String::from("next_server_port"), next_server_port);
        m.insert(String::from("prev_server_ip"), prev_server_ip);
        m.insert(String::from("prev_server_port"), prev_server_port);
        m.clone()
    };
}

async fn run_service(server_addr: &str, port: u16) -> io::Result<()> {
    let parsed_server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = listen(&parsed_server_addr)?;
    let _addr = transport.local_addr();

    let server_id = match HASHMAP.get(&String::from("server_id")) {
        // param was passed
        Some(x) => x.parse::<usize>().unwrap(),
        // no param!
        None => panic!("No input provided for the server_id flag!"),
    };

    let nextaddr: Ipv4Addr = HASHMAP
        .get(&String::from("next_server_ip"))
        .unwrap()
        .parse()
        .unwrap();

    let nextport = HASHMAP
        .get(&String::from("next_server_port"))
        .unwrap()
        .parse::<u16>()
        .unwrap();

    let prevaddr: Ipv4Addr = HASHMAP
        .get(&String::from("prev_server_ip"))
        .unwrap()
        .parse()
        .unwrap();

    let prevport = HASHMAP
        .get(&String::from("prev_server_port"))
        .unwrap()
        .parse::<u16>()
        .unwrap();

    let micro: f64 = match HASHMAP.get(&String::from("micro")) {
        // param was passed
        Some(x) => x.parse::<f64>().unwrap(),
        // no param!
        None => panic!("No input provided for the micro flag!"),
    };

    let scale: f64 = match HASHMAP.get(&String::from("variance")) {
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
        .respond_with(serve(IntermediateServer {
            server_id_arg: server_id,
            next_server_ip: nextaddr,
            next_server_port: nextport,
            prev_server_ip: prevaddr,
            prev_server_port: prevport,
            micro: micro,
            scale: scale,
            forward_arg: false,
        }));

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

    let rpc_service = thread::spawn(move || {
        tokio::run(
            run_service(ip, port)
                .map_err(|e| eprintln!("RPC Error: {}", e))
                .boxed()
                .compat(),
        );
    });

    rpc_service.join().unwrap();
}
