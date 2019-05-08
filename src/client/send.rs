use sharedlib::client_util::{unwrap, wrap};
use sharedlib::head_rpc::new_stub;
use sharedlib::keys::{get, get_keypair, PartyType};
use sharedlib::onion::derive;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::string::String;
use tarpc::{client, context};
use tarpc_bincode_transport::connect;

pub async fn rpc_put(
    server_addr: String,
    port: u16,
    message: String,
    uid: usize,
    remote_uid: usize,
) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&server_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // put this here just to get stuff to work, can fix later
    // TODO: get keys statically
    // get client keypair
    let (priv_key, pub_key) = get_keypair(PartyType::Client.with_id(uid)).unwrap();

    // get other client public key
    let remote_pub_key = get(PartyType::Client.with_id(remote_uid)).unwrap();
    let dk = derive(&priv_key, &remote_pub_key);

    // get round num, this is temporary, actual client that times itself doesnt need this
    let rn = await!(client.getrn(context::current())).unwrap();

    // get vec of server pkeys
    let mut server_pub_keys = vec![];
    server_pub_keys.push(get(PartyType::Server.with_id(0)).unwrap());
    server_pub_keys.push(get(PartyType::Server.with_id(1)).unwrap());
    server_pub_keys.push(get(PartyType::Server.with_id(2)).unwrap());

    let (d_key, enc_msg) = wrap(
        rn,
        message.as_bytes().to_vec(),
        &remote_pub_key,
        &dk,
        &server_pub_keys,
    );
    // store the d_keys for when we receive a message at the end of the round
    // send it
    println!("Encrypted msg: {:?}, len: {:?}", enc_msg, enc_msg.len());
    let return_msg = await!(client.put(context::current(), enc_msg.clone())).unwrap();
    println!("Response: {:?}, len: {:?}", return_msg, return_msg.len());

    if let Ok(unwrapped_msg) = unwrap(rn, return_msg.clone(), &pub_key, &dk, d_key.clone()) {
        let output = String::from_utf8(unwrapped_msg).unwrap();
        println!("{}\n", output);
    } else if let Ok(_) = unwrap(rn, return_msg.clone(), &remote_pub_key, &dk, d_key) {
        println!("Received original message: remote not present.");
    } else {
        println!("Could not decrypt: system error.");
    }

    //t_box.append(output.clone());

    Ok(())
}
