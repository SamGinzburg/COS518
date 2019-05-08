#![allow(non_snake_case)]

use crate::onion;
use std::str;
use std::sync::{Arc, Condvar, Mutex};
use tarpc::context;
use tarpc::futures::future::Ready;
use tarpc::futures::*;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    // buffer for messages received
    pub static ref BACKWARDS_MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    // buffer for messages *after* we process them
    // TODO: clients need to lookup by deaddrop, prob need a HashMap of some kind
    pub static ref PROCESSED_BACKWARDS_MESSAGES: Mutex<Vec<onion::Message>> =
                            Mutex::new(vec![]);
    pub static ref REMOTE_ROUND_ENDED: Arc<(Mutex<bool>, Condvar)> =
                        Arc::new((Mutex::new(false), Condvar::new()));
    // used to block until the round ends
    pub static ref LOCAL_ROUND_ENDED: Arc<(Mutex<bool>, Condvar)> =
                        Arc::new((Mutex::new(false), Condvar::new()));
    pub static ref ROUND_NUM: Mutex<u32> = Mutex::new(0);
}

service! {
    // RPC's for the head server
    rpc put(message: onion::Message) -> onion::Message;
    // this RPC should only be called by the next server in the chain
    rpc SendMessages(v: Vec<onion::Message>) -> bool;
    // this RPC should also only be called by the next server in the chain
    // to signify when it is done sending backwards messages
    rpc EndRound() -> bool;
    // for debugging
    rpc getrn() -> u32;
}

#[derive(Clone, Copy, Debug)]
pub struct HeadServer;

impl self::Service for HeadServer {
    type GetrnFut = Ready<u32>;
    type PutFut = Ready<onion::Message>;
    type SendMessagesFut = Ready<bool>;
    type EndRoundFut = Ready<bool>;

    fn put(self, _: context::Context, s: onion::Message) -> Self::PutFut {
        let msg_count;
        {
            let mut m_vec = MESSAGES.lock().unwrap();
            msg_count = m_vec.len();
            m_vec.push(s.clone());
        }
        println!("DEBUG: incoming msg len: {:?}", s.clone().len());

        // block until the current round ends, send back round reply
        let &(ref b, ref cvar) = &*LOCAL_ROUND_ENDED.clone();
        let mut flag = b.lock().unwrap();
        while !*flag {
            flag = cvar.wait(flag).unwrap();
        }

        let msg_vec = PROCESSED_BACKWARDS_MESSAGES.lock().unwrap();
        println!(
            "DEBUG: Retrieving msg#: {}, total msg count#: {}",
            msg_count,
            msg_vec.len()
        );
        println!("DEBUG: msg len: {:?}", msg_vec[msg_count].clone().len());
        future::ready(msg_vec[msg_count].clone())
    }

    fn SendMessages(self, _: context::Context, v: Vec<onion::Message>) -> Self::SendMessagesFut {
        let mut m_vec = BACKWARDS_MESSAGES.lock().unwrap();
        m_vec.extend(v.clone());
        println!("received messages from next server# = {}", m_vec.len());
        future::ready(true)
    }

    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
        println!("round end called by next server");
        let tuple = REMOTE_ROUND_ENDED.clone();
        let &(ref b, ref cvar) = &*tuple;
        let mut flag = b.lock().unwrap();
        *flag = true;
        cvar.notify_one();
        future::ready(true)
    }

    fn getrn(self, _: context::Context) -> Self::GetrnFut {
        future::ready(*ROUND_NUM.lock().unwrap())
    }
}
