#![feature(futures_api, arbitrary_self_types, await_macro, async_await)]

#[macro_use]
extern crate lazy_static;

extern crate clap;
extern crate rand;
extern crate sharedlib;
extern crate tarpc;
extern crate tarpc_bincode_transport;
extern crate tokio;

mod round;

use crate::tarpc::futures::compat::Executor01CompatExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::TryFutureExt;

use clap::{App, Arg};
use std::collections::HashMap;

use tarpc::server::Handler;
use tarpc_bincode_transport::listen;

use sharedlib::head_rpc::serve;
use sharedlib::head_rpc::HeadServer;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::{io, thread, time};

use crate::round::{
    cleanup, end_round, round_status_check, send_m_vec, start_round, waiting_for_next,
    block_on_replies
};
use sharedlib::head_rpc::{
    BACKWARDS_MESSAGES, LOCAL_ROUND_ENDED, MESSAGES, PROCESSED_BACKWARDS_MESSAGES,
    REQUEST_RESPONSE_BLOCK
};
use tarpc::server;
use std::time::Instant;

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

        let server_uid = String::from(matches.value_of("server_id").unwrap_or("0").clone());
        let server_ip = String::from(matches.value_of("addr").unwrap_or("127.0.0.1").clone());
        let server_port = String::from(matches.value_of("port").unwrap_or("8080").clone());
        let micro = String::from(matches.value_of("micro").unwrap_or("10").clone());
        let b = String::from(matches.value_of("variance").unwrap_or("0").clone());

        m.insert(String::from("variance"), b);
        m.insert(String::from("server_id"), server_uid);
        m.insert(String::from("server_ip"), server_ip);
        m.insert(String::from("server_port"), server_port);
        m.insert(String::from("micro"), micro);

        m.clone()
    };
}

async fn run_service(_server_addr: &str, port: u16) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let transport = listen(&server_addr)?;
    let _addr = transport.local_addr();

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
    let ip = HASHMAP.get(&String::from("server_ip")).unwrap();
    let port = HASHMAP
        .get(&String::from("server_port"))
        .unwrap()
        .parse::<u16>()
        .unwrap();
    
    let handler1 = thread::Builder::new().name("rpc_thread".to_string()).spawn(move || {
       tokio::run(
            run_service(ip, port)
                .map_err(|e| eprintln!("RPC Error: {}", e))
                .boxed()
                .compat(),
        );    
    }).unwrap();


    // start fetching data from server once GUI is initialized
    let handler2 = thread::Builder::new().name("round_thread".to_string()).spawn(move || {
        loop {
            {
                let mut p_backwards_msgs_m_vec = BACKWARDS_MESSAGES.lock().unwrap();
                *p_backwards_msgs_m_vec = vec![];

                // reset the 'messages received' buffer at the start of each round
                let p_backwards_m_vec = PROCESSED_BACKWARDS_MESSAGES.lock();
                let mut unwrapped_backwards = match p_backwards_m_vec {
                    Err(e) => e.into_inner(),
                    Ok(a)  => a
                };
                *unwrapped_backwards = vec![];
            }
            // wait until round ends
            thread::sleep(time::Duration::from_millis(2000));

            // start timing the round
            let now = Instant::now();

            // acquire lock on MESSAGES
            println!("Starting round!!");
            let mut m_vec = MESSAGES.lock().unwrap();
            println!("m_vec lock acquired!");

            let shuffle = round_status_check(m_vec.to_vec(), "127.0.0.1".to_string(), 8081);
            // signal int_server to start round
            let start_new_round =
                shuffle.and_then(|(s, v)| start_round(s, v, "127.0.0.1".to_string(), 8081));
            // begin sending messages in batches
            let send_msgs =
                start_new_round.and_then(|(s, v)| send_m_vec(s, v, "127.0.0.1".to_string(), 8081));
            // signal end of round
            let end_round =
                send_msgs.and_then(|(s, v)| end_round(s, v, "127.0.0.1".to_string(), 8081));
            let wait = end_round.and_then(|(s, _)| waiting_for_next(s));
            let almost_done_cleanup = wait.and_then(|s| cleanup(s));
            //let block_until_replies_done = almost_done_cleanup.and_then(|_| block_on_replies());
    
            tokio::run(
                (almost_done_cleanup)
                    .map_err(|e| eprintln!("Fetch Error: {}", e))
                    .boxed()
                    .compat(),
            );

            tokio::run(
                (block_on_replies())
                    .map_err(|e| eprintln!("Fetch Error: {}", e))
                    .boxed()
                    .compat(),
            );

            // empty our message buffer for the next round
            *m_vec = vec![];
            // cleanup end round cvar
            let tuple = LOCAL_ROUND_ENDED.clone();
            let &(ref b, _) = &*tuple;
            let mut flag = b.lock().unwrap();
            *flag = false;
            // cleanup response handling
            let &(ref b, ref _cvar) = &*REQUEST_RESPONSE_BLOCK.clone();
            let mut flag = b.lock().unwrap();
            *(flag.get_mut()) = 0;
            println!("ROUND TIME ELAPSED (ms): {}", now.elapsed().as_millis());
        }
    }).unwrap();

    handler2.join().unwrap();
    handler1.join().unwrap();
}
