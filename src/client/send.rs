use tarpc::{client, context};
use tarpc_bincode_transport::{connect};
use std::net::{IpAddr, SocketAddr};
use std::{io};
use std::string::String;
use sharedlib::head_rpc::new_stub;
use sharedlib::keys::{PartyType, get, get_keypair};
use sharedlib::onion::derive;
use crate::util::{wrap, unwrap};

pub async fn rpc_put(server_addr: String, port: u16, message: String, uid: usize, remote_uid: usize) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&server_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // put this here just to get stuff to work, can fix later
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

    let (d_keys, enc_msg) = wrap(rn, message.as_bytes().to_vec(), &dk, &server_pub_keys);
    // store the d_keys for when we receive a message at the end of the round
    // send it
    let return_msg = await!(client.put(context::current(), enc_msg.clone())).unwrap();

    let unwrapped_msg = unwrap(rn, return_msg, &dk, d_keys);
    let mut output = String::from_utf8(unwrapped_msg).unwrap();
    output.push_str("\n");
    println!("{}", output);
    //t_box.append(output.clone());

    Ok(())
}
