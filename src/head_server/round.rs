use crate::HASHMAP;
use sharedlib::keys::get_keypair;
use sharedlib::keys::{get, PartyType};
use sharedlib::laplace::{Laplace, TransformedDistribution};
use sharedlib::onion;
use sharedlib::util::{backward, forward, Settings, State};
use std::io;
use std::net::{IpAddr, SocketAddr};
use tarpc::{client, context};
use tarpc_bincode_transport::connect;
// we want to make sure we connect to the intermediate server in our rounds
use sharedlib::head_rpc::{
    BACKWARDS_MESSAGES, LOCAL_ROUND_ENDED, PROCESSED_BACKWARDS_MESSAGES, REMOTE_ROUND_ENDED,
    ROUND_NUM, REQUEST_RESPONSE_BLOCK
};
use sharedlib::int_rpc::new_stub;
use tokio_threadpool::blocking;
use std::time::Instant;

/*
 * This function is used to periodically end a round,
 * flush the messages to the next server in the chain,
 * and begin tracking messages for the next round.
 */
pub async fn round_status_check(
    m_vec: Vec<onion::Message>,
    _server_addr: String,
    _port: u16,
) -> io::Result<(State, Vec<onion::Message>)> {
    //println!("round_status_check");

    let micro: f64 = match HASHMAP.get(&String::from("micro")) {
        // param was passed
        Some(x) => x.parse::<f64>().unwrap(),
        // no param!
        None => panic!("No input provided for the micro flag!"),
    };

    let scale: f64 = match HASHMAP.get(&String::from("variance")) {
        // param was passed
        Some(x) => x.parse::<f64>().unwrap(),
        // no param!
        None => panic!("No input provided for the micro flag!"),
    };

    // permute the messages *before* proceeding further
    let n = Laplace::new(scale, micro);
    let transformed_noise = TransformedDistribution::new(n, |x| u32::max(0, f64::ceil(x) as u32));
    // read in the next two server pub keys
    let mut key_vec = vec![];

    let k1 = match get(PartyType::Server.with_id(1)) {
        Ok(k) => k,
        Err(e) => panic!("Unable to read server public key 1: err: {}", e),
    };
    let k2 = match get(PartyType::Server.with_id(2)) {
        Ok(k) => k,
        Err(e) => panic!("Unable to read server public key 2: err: {}", e),
    };
    key_vec.push(k1);
    key_vec.push(k2);

    let server_id = match HASHMAP.get(&String::from("server_id")) {
        // param was passed
        Some(x) => x.parse::<usize>().unwrap(),
        // no param!
        None => panic!("No input provided for the server_id flag!"),
    };

    let (server_priv_key, _) = match get_keypair(PartyType::Server.with_id(server_id)) {
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
    let _client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // figure out if we need this call at all
    //await!(client.StartNewRound(context::current())).unwrap();
    Ok((s, m_vec))
}

pub async fn send_m_vec(
    s: State,
    m_vec: Vec<onion::Message>,
    server_addr: String,
    port: u16,
) -> io::Result<(State, Vec<onion::Message>)> {
    println!("send_m_vec");
    let s_addr = SocketAddr::new(IpAddr::V4(server_addr.parse().unwrap()), port);
    let transport = await!(connect(&s_addr)).unwrap();
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    // divide the m_vec into evenly sized chunks
    let chunk_count = m_vec.len();
    let m_vec_clone = m_vec.clone();
    let now = Instant::now();
    for count in 0..chunk_count {
        let msgs = m_vec_clone.get(count..count + 1).unwrap().to_vec();
        //println!("sending msgs");
        await!(client.SendMessages(context::current(), msgs, true)).unwrap();
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
    let mut client = await!(new_stub(client::Config::default(), transport)).unwrap();
    await!(client.EndRound(context::current())).unwrap();
    Ok((s, m_vec))
}

pub async fn waiting_for_next(s: State) -> io::Result<State> {
    // after we end the round, we will begin receiving msg's from the int_server
    println!("waiting for intermediate server to finish!");

    blocking(|| {
        // wait int_server signals it is done sending us messages
        let &(ref b, ref cvar) = &*REMOTE_ROUND_ENDED.clone();
        let mut flag = b.lock().unwrap();
        while !*flag {
            flag = cvar.wait(flag).unwrap();
        }
    })
    .map_err(|_| panic!("the threadpool shut down"))
    .unwrap();

    println!("round ended by intermediate server!");
    Ok(s)
}

pub async fn cleanup(s: State) -> io::Result<()> {
    // unshuffle the permutations
    let mut _returning_m_vec = vec![];
    {
        let m_vec = BACKWARDS_MESSAGES.lock();
        match m_vec {
            Err(e) => _returning_m_vec = backward(s, e.into_inner().clone()),
            Ok(v)  => _returning_m_vec = backward(s, v.clone())
        }
    }

    let p_backwards_m_vec = PROCESSED_BACKWARDS_MESSAGES.lock();
    let mut unwrapped_p_backwards_m_vec = match p_backwards_m_vec {
        Err(e) => e.into_inner(),
        Ok(v)  => v
    };
    unwrapped_p_backwards_m_vec.extend(_returning_m_vec);

    // increment round count
    let mut rn = ROUND_NUM.lock().unwrap();
    *rn += 1;
    // reset cond var flag for next round
    {
        let &(ref b, _) = &*REMOTE_ROUND_ENDED.clone();
        let mut flag = b.lock().unwrap();
        *flag = false;
    }
    println!("Round number incremented, now: {}", *rn);

    // send all the messages back!
    let tuple = LOCAL_ROUND_ENDED.clone();
    let &(ref b, ref cvar) = &*tuple;
    let mut flag = b.lock().unwrap();
    *flag = true;
    cvar.notify_all();

    // we need to actually wait for all messages to be flushed!
    println!("waiting to finish replying to all msgs");

    Ok(())
}


pub async fn block_on_replies() -> io::Result<()> {
    let p_backwards_m_vec = PROCESSED_BACKWARDS_MESSAGES.lock();
    let unwrapped_p_backwards_m_vec = match p_backwards_m_vec {
        Err(e) => e.into_inner(),
        Ok(v)  => v
    };
    blocking(|| {
        let &(ref b, ref cvar) = &*REQUEST_RESPONSE_BLOCK.clone();
        let mut flag = b.lock().unwrap();
        println!("Flag: {:?}, len: {:?}", *flag, unwrapped_p_backwards_m_vec.len());
        while *(flag.get_mut()) != unwrapped_p_backwards_m_vec.len() {
            flag = cvar.wait(flag).unwrap();
        }
    })
    .map_err(|_| panic!("the threadpool shut down"))
    .unwrap();
    Ok(())
}