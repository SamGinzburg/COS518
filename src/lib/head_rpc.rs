use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::str;
use std::sync::Mutex;
use crate::onion;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
}

service! {
    // RPC's for the head server
    rpc put(message: onion::Message) -> String;
    rpc get(x: i32, y: i32) -> String;
}

#[derive(Clone, Copy, Debug)]
pub struct HeadServer;

impl self::Service for HeadServer {

    type GetFut = Ready<String>;
    type PutFut = Ready<String>;

    fn put(self, _: context::Context, s: onion::Message) -> Self::PutFut {
        // TODO, store types used in crypto utils, not just a Vec<u8>
        {
            let mut m_vec = MESSAGES.lock().unwrap();
            m_vec.push(s.clone());
            println!("received message# = {}", m_vec.len());
        }
        future::ready(format!("PUT!"))
    }

    fn get(self, _: context::Context, x: i32, y: i32) -> Self::GetFut {
        future::ready(format!("Messages in round = {}", MESSAGES.lock().unwrap().len()))
    }
}


