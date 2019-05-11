#![allow(non_snake_case)]

use crate::keys::get_keypair;
use crate::keys::{get, PartyType};
use crate::laplace::{Laplace, TransformedDistribution};
use crate::util::{backward, forward, Settings, State};
use std::io;
use std::net::{IpAddr, SocketAddr, Ipv4Addr};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use tarpc::futures::future::Ready;
use tarpc::futures::*;
use tarpc::{client, context};
use tarpc_bincode_transport::connect;
use std::time::Instant;

// TODO: need to find a way to dynamically switch which new_stub we use
// depending on flags passed in. Probably by passing down a flag and conditionally
// creating the new_stub.
use crate::deaddrop_rpc::new_stub as next_server_new_stub;
use crate::head_rpc::new_stub as prev_server_new_stub;

// we want to make sure we connect to the intermediate server in our rounds

use crate::onion;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    // buffer for messages received
    pub static ref BACKWARDS_MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    pub static ref REMOTE_ROUND_ENDED: Arc<(Mutex<bool>, Condvar)> =
                        Arc::new((Mutex::new(false), Condvar::new()));
    pub static ref ROUND_NUM: Mutex<u32> = Mutex::new(0);
}

service! {
    // RPC's for the intermediate server
    //
    //  ----------------       ------------------------
    //  | Head Server  |  -->  | Intermediate Server  |
    //  ----------------       ------------------------
    //                    AND
    //  ------------------------      --------------------
    //  | Intermediate Server  | <--  | Deaddrop Server  |
    //  ------------------------      --------------------
    //

    // Head Server ->  Intermediate Server calls
    // tells the server we are done with the curent round
    rpc EndRound() -> bool;
    // Sends a batch of messages in a round
    rpc SendMessages(v: Vec<onion::Message>, is_forward: bool) -> bool;

    // Intermediate Server <- Deaddrop server (or next server in chain)
    // the *next* server in the chain calls this RPC to begin the stage
    // where we send the messages backwards to the previous server in the chain
    rpc EndRoundForward() -> bool;

}

#[derive(Clone, Copy, Debug)]
pub struct IntermediateServer {
    pub server_id_arg: usize,
    pub next_server_ip: Ipv4Addr,
    pub next_server_port: u16,
    pub prev_server_ip: Ipv4Addr,
    pub prev_server_port: u16,
    pub forward_arg: bool,
    pub micro: f64,
    pub scale: f64
}

/*
 * We have to put the async fn calls here, because they are callbacks in response to RPCs
 * as opposed to the head server.
 */

/*
 * This function is used to periodically end a round,
 * flush the messages to the next server in the chain,
 * and begin tracking messages for the next round.
 */
pub async fn round_status_check(
    is: IntermediateServer,
    m_vec: Vec<onion::Message>,
    _server_addr: String,
    _port: u16
) -> io::Result<(State, Vec<onion::Message>)> {
    println!("round_status_check");
    // permute the messages *before* proceeding further
    let n = Laplace::new(is.scale, is.micro);
    let transformed_noise = TransformedDistribution::new(n, |x| u32::max(0, f64::ceil(x) as u32));
    // read in the next server pub keys
    let mut key_vec = vec![];

    // TODO: make this dynamics for n>3 vuvuzela setups
    let k2 = match get(PartyType::Server.with_id(2)) {
        Ok(k) => k,
        Err(e) => panic!("Unable to read server public key 2: err: {}", e),
    };

    key_vec.push(k2);

    let (server_priv_key, _) = match get_keypair(PartyType::Server.with_id(is.server_id_arg)) {
        Ok(kp) => kp,
        Err(e) => panic!("Unable to read server keys!!! err: {}", e),
    };

    println!("shuffling m_vec...");
    let settings = Settings {
        other_pks: key_vec,
        sk: server_priv_key,
        noise: transformed_noise,
    };

    let now = Instant::now();
    let (state, processed_m_vec): (State, Vec<onion::Message>) = forward(m_vec, &settings);
    println!("FORWARD TIME ELAPSED (ms): {}", now.elapsed().as_millis());

    Ok((state, processed_m_vec))
}

pub async fn start_round(
    s: State,
    m_vec: Vec<onion::Message>,
    server_addr: String,
    port: u16,
) -> io::Result<(State, Vec<onion::Message>)> {
    println!("start_round");
    let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&s_addr)).unwrap();
    let _client = await!(next_server_new_stub(client::Config::default(), transport)).unwrap();
    // TODO: figure out if we need this at all
    //await!(client.StartNewRound(context::current())).unwrap();
    Ok((s, m_vec))
}

pub async fn send_m_vec(
    s: State,
    m_vec: Vec<onion::Message>,
    server_addr: String,
    port: u16,
) -> io::Result<(State, Vec<onion::Message>)> {
    println!("forward m_vec");
    let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&s_addr)).unwrap();
    let mut client = await!(next_server_new_stub(client::Config::default(), transport)).unwrap();
    // divide the m_vec into evenly sized chunks
    let chunk_count = m_vec.len();
    let m_vec_clone = m_vec.clone();
    let now = Instant::now();
    for count in 0..chunk_count {
        let msgs = m_vec_clone.get(count..count + 1).unwrap().to_vec();
        await!(client.SendMessages(context::current(), msgs)).unwrap();
    }
    println!("NETWORK FORWARD TIME ELAPSED (ms): {}", now.elapsed().as_millis());

    Ok((s, m_vec))
}

pub async fn end_round(
    s: State,
    m_vec: Vec<onion::Message>,
    server_addr: String,
    port: u16,
) -> io::Result<(State, Vec<onion::Message>)> {
    println!("end_round");

    let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&s_addr)).unwrap();
    let mut client = await!(next_server_new_stub(client::Config::default(), transport)).unwrap();
    await!(client.EndRound(context::current())).unwrap();
    Ok((s, m_vec))
}

pub async fn cleanup(
    s: State,
    m_vec: Vec<onion::Message>,
    _server_addr: String,
    _port: u16,
) -> io::Result<Vec<onion::Message>> {
    // unshuffle the permutations
    Ok(backward(s, m_vec))
}

// send messages to previous server finally & finish cleanup
pub async fn wait_for_reply(s: State) -> io::Result<(State, Vec<onion::Message>)> {
    // wait int_server signals it is done sending us messages
    println!("waiting on the next server to finish sending msgs");
    let &(ref b, ref cvar) = &*REMOTE_ROUND_ENDED.clone();
    let mut flag = b.lock().unwrap();
    while !*flag {
        flag = cvar.wait(flag).unwrap();
    }
    // reset cond var flag for next round
    *flag = false;
    println!("round ended by the next server!");

    Ok((s, BACKWARDS_MESSAGES.lock().unwrap().to_vec()))
}

// send messages to previous server finally & finish cleanup
pub async fn backwards_send_msg(
    m_vec: Vec<onion::Message>,
    server_addr: String,
    port: u16,
) -> io::Result<()> {
    println!("backwards_send_msg");

    let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&s_addr)).unwrap();
    let mut client = await!(prev_server_new_stub(client::Config::default(), transport)).unwrap();

    // send all the messages
    let chunk_count = m_vec.len();
    println!("{}", chunk_count);
    let m_vec_clone = m_vec.clone();

    let now = Instant::now();
    for count in 0..chunk_count {
        //println!("sending message backwards!");
        let msgs = m_vec_clone.get(count..count + 1).unwrap().to_vec();
        //println!("msg len: {:?}", msgs[0].len());
        await!(client.SendMessages(context::current(), msgs)).unwrap();
    }
    println!("NETWORK FORWARD TO HEAD TIME ELAPSED (ms): {}", now.elapsed().as_millis());

    // increment round count
    let mut rn = ROUND_NUM.lock().unwrap();
    *rn += 1;
    println!("Round number incremented, now: {}", *rn);
    // empty MESSAGES
    let mut msgs = MESSAGES.lock().unwrap();
    *msgs = vec![];
    let mut back_msgs = BACKWARDS_MESSAGES.lock().unwrap();
    *back_msgs = vec![];

    Ok(())
}

pub async fn backwards_end_round(server_addr: String, port: u16) -> io::Result<()> {
    println!("ending round on previous server");

    let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&s_addr)).unwrap();
    let mut client = await!(prev_server_new_stub(client::Config::default(), transport)).unwrap();
    await!(client.EndRound(context::current())).unwrap();

    Ok(())
}

impl self::Service for IntermediateServer {
    type EndRoundFut = Ready<bool>;
    type EndRoundForwardFut = Ready<bool>;
    type SendMessagesFut = Ready<bool>;

    // next server calls this to end the round and begin sending backwards
    fn EndRoundForward(self, _: context::Context) -> Self::EndRoundForwardFut {
        let tuple = REMOTE_ROUND_ENDED.clone();
        let &(ref b, ref cvar) = &*tuple;
        let mut flag = b.lock().unwrap();
        *flag = true;
        cvar.notify_one();

        future::ready(true)
    }

    // head server calls this to signify when it is done
    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
        // this is the trigger to spin off a thread to forward all messages
        // to the next server
        let _rpc_service = thread::spawn(move || {
            let m_vec = MESSAGES.lock().unwrap();
            let copy_m_vec = m_vec.to_vec();
            drop(m_vec);
            let next_ip = self.next_server_ip;
            let next_port = self.next_server_port.clone();
            let prev_port = self.prev_server_port.clone();

            let shuffle = round_status_check(self, copy_m_vec, next_ip.to_string(), next_port);
            // signal int_server to start round
            let start_new_round =
                shuffle.and_then(move |(s, v)| start_round(s, v, next_ip.to_string(), next_port.clone()));
            // begin sending messages in batches
            let send_msgs =
                start_new_round.and_then(move |(s, v)| send_m_vec(s, v, next_ip.to_string(), next_port.clone()));
            // signal end of round
            let end_round =
                send_msgs.and_then(move |(s, v)| end_round(s, v, next_ip.to_string(), next_port.clone()));
            let wait = end_round.and_then(|(s, _)| wait_for_reply(s));
            let backwards_permute =
                wait.and_then(move |(s, v)| cleanup(s, v, self.prev_server_ip.to_string(), prev_port.clone()));
            // only after the next server is done, can we start sending msgs back
            let respond = backwards_permute
                .and_then(move |v| backwards_send_msg(v, self.prev_server_ip.to_string(), prev_port));
            let end_previous =
                respond.and_then(move |_| backwards_end_round(self.prev_server_ip.to_string(), prev_port));

            tokio::run(
                (end_previous)
                    .map_err(|e| eprintln!("RPC Error: {}", e))
                    .boxed()
                    .compat(),
            );
        });
        future::ready(true)
    }

    // only the head server calls this RPC
    fn SendMessages(
        self,
        _: context::Context,
        v: Vec<onion::Message>,
        is_forward: bool,
    ) -> Self::SendMessagesFut {
        if is_forward {
            let mut m_vec = MESSAGES.lock().unwrap();
            m_vec.extend(v.clone());
            //println!("# messages received from prev = {}", m_vec.len());
        } else {
            let mut m_vec = BACKWARDS_MESSAGES.lock().unwrap();
            m_vec.extend(v.clone());
            //println!("# messages received from next = {}", m_vec.len());
        }
        future::ready(true)
    }
}
