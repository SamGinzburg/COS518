use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::str;
use std::sync::{Arc, Mutex, Condvar};
use crate::onion;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    // buffer for messages received
    pub static ref BACKWARDS_MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    // buffer for messages *after* we process them
    pub static ref PROCESSED_BACKWARDS_MESSAGES: Mutex<Vec<onion::Message>> = 
                            Mutex::new(vec![]);
    pub static ref REMOTE_ROUND_ENDED: Arc<(Mutex<bool>, Condvar)> = 
                        Arc::new((Mutex::new(false), Condvar::new()));
    pub static ref ROUND_NUM: Mutex<u32> = Mutex::new(0);
}

service! {
    // RPC's for the head server
    rpc put(message: onion::Message) -> String;
    // this RPC should only be called by the next server in the chain
    rpc putnext(message: onion::Message) -> String;
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
    type PutFut = Ready<String>;
    type PutnextFut = Ready<String>;
    type EndRoundFut = Ready<bool>;

    fn put(self, _: context::Context, s: onion::Message) -> Self::PutFut {
        let mut m_vec = MESSAGES.lock().unwrap();
        m_vec.push(s.clone());
        println!("received message# = {}", m_vec.len());
        future::ready(format!("PUT!"))
    }

    fn putnext(self, _: context::Context, s: onion::Message) -> Self::PutFut {
        let mut m_vec = BACKWARDS_MESSAGES.lock().unwrap();
        m_vec.push(s.clone());
        println!("received message from next server# = {}", m_vec.len());
        future::ready(format!("PUT!"))
    }

    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
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


