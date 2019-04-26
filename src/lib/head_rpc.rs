use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::str;
use std::sync::Mutex;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<Vec<u8>>> = Mutex::new(vec![]);
}

service! {
    // RPC's for the head server
    rpc put(message: Vec<u8>) -> String;
    rpc get(x: i32, y: i32) -> String;
}

#[derive(Clone, Copy, Debug)]
pub struct HeadServer;

impl self::Service for HeadServer {

    type GetFut = Ready<String>;
    type PutFut = Ready<String>;

    fn put(self, _: context::Context, s: Vec<u8>) -> Self::PutFut {
        // TODO, store types used in crypto utils, not just a Vec<u8>
        MESSAGES.lock().unwrap().push(s.clone());
        println!("received message# = {}", MESSAGES.lock().unwrap().len());
        future::ready(format!("PUT, {}!", str::from_utf8(&s).unwrap()))
    }

    fn get(self, _: context::Context, x: i32, y: i32) -> Self::GetFut {
        future::ready(format!("Messages in round = {}", MESSAGES.lock().unwrap().len()))
    }
}


