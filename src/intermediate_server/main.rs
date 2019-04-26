#![feature(futures_api, arbitrary_self_types, await_macro, async_await)]

#[macro_use] extern crate tarpc;

extern crate rand;
extern crate clap;
extern crate tarpc_bincode_transport;
extern crate tokio;
extern crate sharedlib;

use crate::tarpc::futures::StreamExt;
use crate::tarpc::futures::TryFutureExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::compat::Executor01CompatExt;

use clap::{App};


use tarpc::{
    server::{Handler},
};
use tarpc_bincode_transport::listen;

use sharedlib::int_rpc::IntermediateServer;
use sharedlib::int_rpc::serve;

use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use tarpc::server;

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
        .respond_with(serve(IntermediateServer));

    await!(server);
    
    Ok(())
}

fn main() {
    App::new("Vuvuzela Server")
         .version("1.0")
         .about("Vuvuzela Server")
         .author("Sam Ginzburg")
         .author("Benjamin Kuykendall")
         .get_matches();

    tarpc::init(tokio::executor::DefaultExecutor::current().compat());
    // TODO: set ip/port combo via cli flags
    tokio::run(run_service("", 8080)
               .map_err(|e| eprintln!("RPC Error: {}", e))
               .boxed()
               .compat(),
    );
}
