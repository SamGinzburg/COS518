use tarpc::{client, context, server};
use tarpc_bincode_transport::{connect, listen, Transport};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::io;
use futures::Stream;
use sharedlib::head_rpc::new_stub;
use sharedlib::onion::wrap;
use crate::HASHMAP;
use sharedlib::keys::{Party, PartyType, get, get_keypair};

pub async fn rpc_put(server_addr: String, port: u16, message: String, uid: usize, remote_uid: usize) -> io::Result<()> {
    let server_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&server_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // put this here just to get stuff to work, can fix later
    // get client keypair
    let (priv_key, pub_key) = get_keypair(PartyType::Client.with_id(uid)).unwrap();
    // get server public key
    let server_pub_key = get(PartyType::Server.with_id(remote_uid)).unwrap();

    // packup the message
    /* fn wrap(
    round : u32,
    mut m : Vec<u8>,
    dk : &onion::DerivedKey,
    server_pks : &Vec<onion::PublicKey>,
) -> (Vec<onion::DerivedKey>, onion::Message) */
    //let enc_msg = wrap (0, &message.as_bytes().to_vec(), )
    // send it
    //let response = await!(client.put(context::current(), enc_msg)).unwrap();
    // debug response
    //println!("{}", response);

    Ok(())
}
