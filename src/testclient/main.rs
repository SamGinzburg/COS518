#![feature(
    type_ascription,
    generators,
    proc_macro_hygiene,
    futures_api,
    arbitrary_self_types,
    await_macro,
    async_await
)]
#[macro_use]
extern crate lazy_static;

extern crate clap;
extern crate cursive;
extern crate futures;
extern crate futures_await_async_macro;
extern crate serde;
extern crate sharedlib;
extern crate tarpc;
extern crate tarpc_bincode_transport;
extern crate tokio;

use clap::{App, Arg};
use std::collections::HashMap;

use crate::send::rpc_put;
use crate::tarpc::futures::compat::Executor01CompatExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::TryFutureExt;
use std::{thread};
use std::io;
use sharedlib::keys::{get, get_keypair, PartyType};
use sharedlib::onion::{PublicKey, PrivateKey, DerivedKey};
use std::time::Duration;
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::AtomicUsize;

use tokio::executor::thread_pool;
use tokio::executor::thread_pool::Builder;

pub mod fetch;
pub mod send;

lazy_static! {
    pub static ref BLOCK: Arc<(Mutex<AtomicUsize>, Condvar)> =
                        Arc::new((Mutex::new(AtomicUsize::new(0)), Condvar::new()));

    // quick hack to get args into callback function without modifying the
    // cursive lib / making a custom UI object
    static ref HASHMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        let matches = App::new("Vuvuzela Client")
                        .version("1.0")
                        .about("Vuvuzela Client")
                        .author("Sam Ginzburg")
                        .author("Benjamin Kuykendall")
                        // this effectively corresponds to the id # for key lookup
                        .arg(Arg::with_name("name")
                            .short("n")
                            .long("name")
                            .help("Specifies the name of the user (used to lookup public/private key pair)")
                            .takes_value(true))
                        .arg(Arg::with_name("dial")
                            .short("d")
                            .long("dial")
                            .help("Specifies the name of the person you are dialing")
                            .takes_value(true))
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
                        .get_matches();

        // if these unwraps fail, we must panic!
        let uid = match matches.value_of("name") {
            Some(u) => String::from(u).clone(),
            None    => panic!("name not specified in CLI arguments"),
        };


        let remote_uid = match matches.value_of("dial") {
            Some(u) => String::from(u).clone(),
            None    => panic!("dial not specified in CLI arguments"),
        };

        let server_ip = String::from(matches.value_of("addr").unwrap_or("127.0.0.1").clone());
        let server_port = String::from(matches.value_of("port").unwrap_or("8080").clone());

        m.insert(String::from("server_ip"), server_ip);
        m.insert(String::from("server_port"), server_port);
        m.insert(String::from("uid"), uid);
        m.insert(String::from("remote_uid"), remote_uid);
        m.clone()
    };

    static ref keypair: (PrivateKey, PublicKey) = { 
    let uid = HASHMAP
        .get(&String::from("uid"))
        .unwrap()
        .parse::<usize>()
        .unwrap();
        get_keypair(PartyType::Client.with_id(uid)).unwrap() 
    };

    static ref remote_pub_key: PublicKey = { 
    let remote_uid = HASHMAP
        .get(&String::from("remote_uid"))
        .unwrap()
        .parse::<usize>()
        .unwrap();
        get(PartyType::Client.with_id(remote_uid)).unwrap()
    };

    static ref MY_PRIV_KEY: PrivateKey = { 
        let uid = HASHMAP
            .get(&String::from("uid"))
            .unwrap()
            .parse::<usize>()
            .unwrap();
        let (priv_key, _) = get_keypair(PartyType::Client.with_id(uid)).unwrap();
        priv_key
    };

    static ref server_pub_keys: Vec<PublicKey> = {
        let mut spk = vec![];
        spk.push(get(PartyType::Server.with_id(0)).unwrap());
        spk.push(get(PartyType::Server.with_id(1)).unwrap());
        spk.push(get(PartyType::Server.with_id(2)).unwrap());
        spk
    };
}

pub async fn spawn_many(
    thread_id: usize,
    remote_uid: usize
) -> io::Result<()> {
    let uid = HASHMAP
        .get(&String::from("uid"))
        .unwrap()
        .parse::<usize>()
        .unwrap();

    let ip = HASHMAP.get(&String::from("server_ip")).unwrap();
    let port = HASHMAP
        .get(&String::from("server_port"))
        .unwrap()
        .parse::<u16>()
        .unwrap();


    await!(rpc_put(
        ip.to_string(),
        port,
        String::from(""),
        uid,
        remote_uid,
        thread_id
    )).unwrap();


    Ok(())
}

fn main() {
    tarpc::init(tokio::executor::DefaultExecutor::current().compat());
    let mut threads = vec![];
    // parallel threads
    for x in 0..1 {
            for y in 1..1000 {
                let handler = thread::spawn(move || {
                    tokio::run(
                            spawn_many(x, y)
                            .map_err(|e| eprintln!("RPC Error: {}", e))
                            .boxed()
                            .compat(),
                    );
                });
                threads.push(handler);
            }
    }

    /*
    let &(ref b, ref cvar) = &*BLOCK.clone();
    let mut flag = b.lock().unwrap();
    println!("Flag: {:?}, len: {:?}", *flag, 500);
    while *(flag.get_mut()) != 500 {
        flag = cvar.wait(flag).unwrap();
    }
    */
    for x in threads {
        x.join();
    }
    //thread_pool.shutdown_on_idle();
}
