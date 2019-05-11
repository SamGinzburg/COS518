#![allow(non_snake_case)]

use crate::onion;
use std::str;
use std::sync::{Arc, Condvar, Mutex};
use tarpc::context;
use tarpc::futures::future::Ready;
use tarpc::futures::*;
use tokio_threadpool::blocking;
use std::sync::atomic::AtomicUsize;

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
    // used to block until the round ends
    pub static ref LOCAL_ROUND_ENDED: Arc<(Mutex<bool>, Condvar)> =
                        Arc::new((Mutex::new(false), Condvar::new()));
    // block until we finish replying to everyone
    pub static ref REQUEST_RESPONSE_BLOCK: Arc<(Mutex<AtomicUsize>, Condvar)> =
                        Arc::new((Mutex::new(AtomicUsize::new(0)), Condvar::new()));
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
        //println!("DEBUG: incoming msg len: {:?}", s.clone().len());

        // block until the current round ends, send back round reply

        blocking(|| {
            let &(ref b, ref cvar) = &*LOCAL_ROUND_ENDED.clone();
            let mut flag = b.lock().unwrap();
            while !*flag {
                let (f, _) = cvar.wait_timeout_ms(flag, 200).unwrap();
                flag = f;
                //println!("waiting for end of round: count: {:?}", flag);
            }
        })
        .map_err(|_| {
            println!("unable to block!"); panic!("the threadpool shut down")
        })
        .unwrap();

        let temp = PROCESSED_BACKWARDS_MESSAGES.lock();
        let msg_vec = match temp {
            Err(e) => e.into_inner(),
            Ok(o)  => o
        };

        /*println!(
            "DEBUG: Retrieving msg#: {}, total msg count#: {}",
            msg_count,
            msg_vec.len()
        );*/

        let tuple = REQUEST_RESPONSE_BLOCK.clone();
        let &(ref b, ref cvar) = &*tuple;
        let mut flag = match b.lock() {
            Err(e) => e.into_inner(),
            Ok(o)  => o
        };
        *flag.get_mut() += 1;
        cvar.notify_one();

        //println!("DEBUG: msg len: {:?}", msg_vec[msg_count].clone().len());
        future::ready(msg_vec[msg_count].clone())
    }

    fn SendMessages(self, _: context::Context, v: Vec<onion::Message>) -> Self::SendMessagesFut {
        blocking(|| {
            let mut m_vec = BACKWARDS_MESSAGES.lock();
            let mut b_msgs = match m_vec {
                Err(e) => e.into_inner(),
                Ok(a)  => a
            };
            b_msgs.extend(v.clone());
            //println!("received messages from next server# = {}", b_msgs.len());
        })
        .map_err(|_| panic!("the threadpool shut down"))
        .unwrap();
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
