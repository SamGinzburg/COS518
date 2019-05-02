use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::str;
use std::sync::Mutex;
use crate::onion;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
    pub static ref ROUND_NUM: Mutex<u32> = Mutex::new(0);
}

service! {
    // RPC's for the head server
    rpc put(message: onion::Message) -> String;
    // for debugging
    rpc getrn() -> u32;
}

#[derive(Clone, Copy, Debug)]
pub struct HeadServer;

impl self::Service for HeadServer {

    type GetrnFut = Ready<u32>;
    type PutFut = Ready<String>;

    fn put(self, _: context::Context, s: onion::Message) -> Self::PutFut {
        let mut m_vec = MESSAGES.lock().unwrap();
        m_vec.push(s.clone());
        println!("received message# = {}", m_vec.len());
        future::ready(format!("PUT!"))
    }

    fn getrn(self, _: context::Context) -> Self::GetrnFut {
        future::ready(*ROUND_NUM.lock().unwrap())
    }
}


