use sharedlib::head_rpc::MESSAGES;
use sharedlib::{message, onion};
use sharedlib::laplace::{Laplace, TransformedDistribution};
use sharedlib::util::{forward, Settings, State};
use std::{thread, time, io};
use sharedlib::keys::{Party, PartyType, get};
use crate::HASHMAP;
use sharedlib::keys::get_keypair;

/*
 * This function is used to periodically end a round,
 * flush the messages to the next server in the chain,
 * and begin tracking messages for the next round.
 */
pub async fn round_status_check() -> io::Result<()> {
	// acquire lock on MESSAGES
	let m_vec = MESSAGES.lock().unwrap();
	// permute the messages *before* proceeding further
	// fn forward<D>(input : Vec<onion::Message>, settings : &Settings<D>)-> (State, Vec<onion::Message>)
	// where D : Distribution<u32> {
	let n: Laplace = Laplace::new(1.0_f64, 1.0_f64);
	let transformed_noise = TransformedDistribution::new(n, |x| u32::max(0, f64::ceil(x) as u32));
	// read in the next two server pub keys
	let mut key_vec = vec![];
	key_vec.push(get(PartyType::Server.with_id(1)).unwrap());
	key_vec.push(get(PartyType::Server.with_id(2)).unwrap());

	let server_id = HASHMAP.get(&String::from("server_id")).unwrap().parse::<usize>().unwrap();
	let (server_priv_key, _) = get_keypair(PartyType::Server.with_id(server_id)).unwrap();

	let settings = Settings{ other_pks: key_vec, sk: server_priv_key, noise: transformed_noise };
	let (state, processed_m_vec): (State, Vec<onion::Message>) = forward(m_vec.to_vec(), &settings);
	// start a new round on the next server
	// begin sending messages to the intermediate server in chunks
	// end the round via RPC to next server (this triggers the next server to start forwarding)
	// wait for replies from server 2, once server 2 receives its own replies
	// increment the round number
	// clear the message vector
	// when we are done drop the lock on MESSAGES by ending the scope
	Ok(())
}