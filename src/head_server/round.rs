use tarpc::{client, context};
use tarpc_bincode_transport::{connect};
use std::net::{IpAddr, SocketAddr};
use sharedlib::{onion};
use sharedlib::laplace::{Laplace, TransformedDistribution};
use sharedlib::util::{backward, forward, Settings, State};
use std::{io};
use sharedlib::keys::{PartyType, get};
use crate::HASHMAP;
use sharedlib::keys::get_keypair;
// we want to make sure we connect to the intermediate server in our rounds
use sharedlib::int_rpc::new_stub;
use sharedlib::head_rpc::{ROUND_NUM, REMOTE_ROUND_ENDED, PROCESSED_BACKWARDS_MESSAGES};

/*
 * This function is used to periodically end a round,
 * flush the messages to the next server in the chain,
 * and begin tracking messages for the next round.
 */
pub async fn round_status_check(m_vec: Vec<onion::Message>, server_addr: String, port: u16)
-> io::Result<(State, Vec<onion::Message>)> {
	println!("round_status_check");
	// permute the messages *before* proceeding further
	let n: Laplace = Laplace::new(1.0_f64, 1.0_f64);
	let transformed_noise = TransformedDistribution::new(n, |x| u32::max(0, f64::ceil(x) as u32));
	// read in the next two server pub keys
	let mut key_vec = vec![];

	let k1 = match get(PartyType::Server.with_id(1)) {
		Ok(k)  => k,
		Err(e) => panic!("Unable to read server public key 1: err: {}", e),
	};
	let k2 = match get(PartyType::Server.with_id(2)) {
		Ok(k)  => k,
		Err(e) => panic!("Unable to read server public key 2: err: {}", e),
	};
	key_vec.push(k1);
	key_vec.push(k2);

	let server_id = match HASHMAP.get(&String::from("server_id")) {
		// param was passed
		Some(x) => x.parse::<usize>().unwrap(),
		// no param!
		None    => panic!("No input provided for the server_id flag!"),
	};
	
	let (server_priv_key, _) = match get_keypair(PartyType::Server.with_id(server_id)) {
		Ok(kp) => kp,
		Err(e) => panic!("Unable to read server keys!!! err: {}", e),
	};

	println!("shuffling m_vec...");
	let settings = Settings{ other_pks: key_vec, sk: server_priv_key, noise: transformed_noise };
	let (state, processed_m_vec): (State, Vec<onion::Message>) = forward(m_vec, &settings);

	Ok((state, processed_m_vec))
}

pub async fn start_round(s: State, m_vec: Vec<onion::Message>, server_addr: String, port: u16) 
-> io::Result<(State, Vec<onion::Message>)> {
	println!("start_round");
	let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
	let transport = await!(connect(&s_addr)).unwrap();
	let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
	// figure out if we need this call at all
	//await!(client.StartNewRound(context::current())).unwrap();
	Ok((s, m_vec))
}

pub async fn send_m_vec(s: State, m_vec: Vec<onion::Message>, server_addr: String, port: u16)
-> io::Result<(State, Vec<onion::Message>)> {
	println!("send_m_vec");
	let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
	let transport = await!(connect(&s_addr)).unwrap();
	let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
	// divide the m_vec into evenly sized chunks
	let chunk_count = m_vec.len();
	let m_vec_clone = m_vec.clone();
	for count in 0..chunk_count {
		let msgs = m_vec_clone.get(count..count+1).unwrap().to_vec();
		await!(client.SendMessages(context::current(), msgs, true)).unwrap();
	}

	Ok((s, m_vec))
}

pub async fn end_round(s: State, m_vec: Vec<onion::Message>, server_addr: String, port: u16)
-> io::Result<(State, Vec<onion::Message>)> {
	println!("end_round");

	let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
	let transport = await!(connect(&s_addr)).unwrap();
	let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
	await!(client.EndRound(context::current())).unwrap();
	Ok((s, m_vec))
}


pub async fn cleanup(s: State, m_vec: Vec<onion::Message>)
-> io::Result<()> {
	// after we end the round, we will begin receiving msg's from the int_server
	println!("waiting for intermediate server to finish!");

	// wait int_server signals it is done sending us messages
	let &(ref b, ref cvar) = &*REMOTE_ROUND_ENDED.clone();
	let mut flag = b.lock().unwrap();
	while !*flag {
		flag = cvar.wait(flag).unwrap();
	}

	println!("round ended by intermediate server!");
	// unshuffle the permutations
	let mut p_backwards_m_vec = PROCESSED_BACKWARDS_MESSAGES.lock().unwrap();
	let returning_m_vec = backward(s, m_vec);
	p_backwards_m_vec.extend(returning_m_vec);

	// increment round count
	let mut rn = ROUND_NUM.lock().unwrap();
	*rn += 1;
	// reset cond var flag for next round
	*flag = false;
	println!("Round number incremented, now: {}", *rn);

	Ok(())
}