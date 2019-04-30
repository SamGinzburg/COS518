#![feature(futures_api, arbitrary_self_types, await_macro, async_await)]

#[macro_use] extern crate tarpc;
#[macro_use] extern crate lazy_static;

extern crate rand;
extern crate clap;
extern crate tarpc_bincode_transport;
extern crate tokio;
extern crate sharedlib;

mod round;

use crate::tarpc::futures::StreamExt;
use crate::tarpc::futures::TryFutureExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::compat::Executor01CompatExt;

use clap::{App, Arg};
use std::collections::HashMap;

use tarpc::{
    server::{Handler},
};
use tarpc_bincode_transport::listen;

use sharedlib::head_rpc::HeadServer;
use sharedlib::head_rpc::serve;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{thread, time, io};

use tarpc::server;
use std::sync::Mutex;
use crate::round::round_status_check;


lazy_static! {
    // quick hack to get args into callback function without modifying the 
    // cursive lib / making a custom UI object
    static ref HASHMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        let matches = App::new("Vuvuzela Head Server")
                        .version("1.0")
                        .about("Vuvuzela Head Server")
                        .author("Sam Ginzburg")
                        .author("Benjamin Kuykendall")
                        // this effectively corresponds to the id # for key lookup
                        .arg(Arg::with_name("server_id")
                            .short("id")
                            .long("server_id")
                            .help("Specifies which server keypair to use")
                            .takes_value(true))
                        .get_matches();

        let server_uid = String::from(matches.value_of("server_id").unwrap().clone());

        m.insert(String::from("server_id"), server_uid);
        m.clone()
    };
}


async fn run_service(server_addr: &str, port: u16) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let transport = listen(&server_addr)?;
    let addr = transport.local_addr();

    // The server is configured with the defaults.
    let server = server::new(server::Config::default())
        // Server can listen on any type that implements the Transport trait.
        .incoming(transport)
        // serve is generated by the service! macro. It takes as input any type implementing
        // the generated Service trait.
        .respond_with(serve(HeadServer));

    await!(server);
    
    Ok(())
}

fn main() {

    tarpc::init(tokio::executor::DefaultExecutor::current().compat());
    // TODO: set ip/port combo via cli flags
    tokio::run(run_service("", 8080)
               .map_err(|e| eprintln!("RPC Error: {}", e))
               .boxed()
               .compat(),
    );


    // start fetching data from server once GUI is initialized
    let handler = thread::spawn(|| {
        loop {
            tokio::run((round_status_check())
                    .map_err(|e| {
                        eprintln!("Fetch Error: {}", e) })
                    .boxed()
                    .compat(),);
            thread::sleep(time::Duration::from_millis(1000));
        }
    });
}
