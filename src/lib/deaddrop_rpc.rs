#![allow(non_snake_case)]

use tarpc::futures::*;
use tarpc::futures::future::Ready;
use std::str;
use crate::onion;
use crate::util::deaddrop;
use tarpc::{client, context};
use tarpc_bincode_transport::{connect};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Mutex};
use std::thread;
use crate::int_rpc::new_stub as prev_server_stub;
use crate::util::{backward, forward, Settings, State};
use crate::keys::get_keypair;
use crate::laplace::{Laplace, TransformedDistribution};
use crate::keys::{PartyType};

lazy_static! {
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    pub static ref ROUND_NUM: Mutex<u32> = Mutex::new(0);
}

pub async fn send_m_vec(m_vec: Vec<onion::Message>, server_addr: String, port: u16)
-> io::Result<()> {
	println!("respond with swapped m_vec");
	let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
	let transport = await!(connect(&s_addr)).unwrap();
	let mut client = await!(prev_server_stub(client::Config::default(), transport)).unwrap();
	// divide the m_vec into evenly sized chunks
	let chunk_count = m_vec.len();
	let m_vec_clone = m_vec.clone();
	for count in 0..chunk_count {
		let msgs = m_vec_clone.get(count..count+1).unwrap().to_vec();
		await!(client.SendMessages(context::current(), msgs, false)).unwrap();
	}

	Ok(())
}


pub async fn end_round(server_addr: String, port: u16)
-> io::Result<()> {
	println!("respond with swapped m_vec");
	let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
	let transport = await!(connect(&s_addr)).unwrap();
	let mut client = await!(prev_server_stub(client::Config::default(), transport)).unwrap();
    await!(client.EndRoundForward(context::current())).unwrap();
    let mut m_vec = MESSAGES.lock().unwrap();
    *m_vec = vec![];
	Ok(())
}

pub async fn forward_fn(m_vec: Vec<onion::Message>)
-> io::Result<(State, Vec<onion::Message>)> {
	println!("forwarding...");
	let n = Laplace::new(1.0, 10.0);
	let transformed_noise = TransformedDistribution::new(n, |x| u32::max(0, f64::ceil(x) as u32));
    let key_vec = vec![];
    let (server_priv_key, _) = match get_keypair(PartyType::Server.with_id(2)) {
		Ok(kp) => kp,
		Err(e) => panic!("Unable to read server keys!!! err: {}", e),
	};
	let settings = Settings{ other_pks: key_vec, sk: server_priv_key, noise: transformed_noise };
	Ok(forward(m_vec, &settings))
}

pub async fn dead_drop_fn(st: State, m_vec: Vec<onion::Message>)
-> io::Result<(State, Vec<onion::Message>)> {
	println!("swapping deaddrops...");
	Ok((st, deaddrop(m_vec)))
}

pub async fn backwards_fn(st: State, m_vec: Vec<onion::Message>)
-> io::Result<(Vec<onion::Message>)> {
	println!("sending msgs backwards...");
	Ok(backward(st, m_vec))
}

service! {
    // RPC's for the intermediate server
    //
    //  ----------------------      ---------------------
    //  | Intermediate Server | ->  | Dead Drop Server  |
    //  ----------------------      ---------------------
    //                    AND
    //  ------------------------      ---------------------
    //  | Intermediate Server  | <--  | Dead Drop Server  |
    //  ------------------------      ---------------------
    //
    // TODO
    rpc EndRound() -> bool;
    // Sends a batch of messages in a round
    rpc SendMessages(v: Vec<onion::Message>) -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct DeadDropServer;

impl self::Service for DeadDropServer {
    type EndRoundFut = Ready<bool>;
    type SendMessagesFut = Ready<bool>;

    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
        // when the round is ended, send everything backwards to the previous server
        // in the chain

        let _rpc_service = thread::spawn(move || {
            let m_vec = MESSAGES.lock().unwrap();
            let m_vec_copy = m_vec.to_vec();
            drop(m_vec);
            let fwd = forward_fn(m_vec_copy);
            let dd = fwd.and_then(|(s, m)| dead_drop_fn(s, m));
            let bwd = dd.and_then(|(s, m)| backwards_fn(s, m));
            let send = bwd.and_then(|v| send_m_vec(v, "127.0.0.1".to_string(), 8081));
            let end = send.and_then(|_| end_round("127.0.0.1".to_string(), 8081));
            tokio::run((end)
                    .map_err(|e| eprintln!("RPC Error: {}", e))
                    .boxed()
                    .compat(),
            );
        });

        future::ready(true)
    }

    fn SendMessages(self, _: context::Context, v: Vec<onion::Message>) -> Self::SendMessagesFut {
        println!("messages arriving to the deaddrop!");
        let mut m_vec = MESSAGES.lock().unwrap();
        m_vec.extend(v.clone());
        println!("mvec size: {}!", m_vec.len());

        future::ready(true)
    }
}

