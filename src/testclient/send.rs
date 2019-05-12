use sharedlib::client_util::wrap;
use sharedlib::head_rpc::new_stub;
use sharedlib::onion::{derive};
use sharedlib::keys::{get, PartyType};

use std::io;
use std::net::{IpAddr, SocketAddr};
use std::string::String;
use tarpc::{client, context};
use tarpc_bincode_transport::connect;
use std::time::Instant;
use tokio_threadpool::blocking;

use crate::{server_pub_keys, MY_PRIV_KEY, BLOCK};

pub async fn rpc_put(
    server_addr: String,
    port: u16,
    message: String,
    uid: usize,
    remote_uid: usize,
    thread_id: usize,
) -> io::Result<()> {
    //println!("running async");
    let server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&server_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // get round num, this is temporary, actual client that times itself doesnt need this
    //let rn = await!(client.getrn(context::current())).unwrap();
    let rn = 0;
    // get vec of server pkeys
    let rpk = get(PartyType::Client.with_id(remote_uid * thread_id)).unwrap();
    let dk = derive(&MY_PRIV_KEY, &rpk);

    let (_, enc_msg) = wrap(
        rn,
        message.as_bytes().to_vec(),
        &rpk,
        &dk,
        &server_pub_keys,
    );

    let now = Instant::now();
    let _return_msg = await!(client.put(context::current(), enc_msg.clone())).unwrap();
    println!("{}", now.elapsed().as_millis());
    Ok(())
}
