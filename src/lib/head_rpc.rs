use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::str;

service! {
    // RPC's for the head server
    rpc put(message: Vec<u8>) -> String;
    rpc get(x: i32, y: i32) -> String;
    // RPC's for the intermediate server
    // TODO
    // RPC's for the deaddrop server
    // TODO
}

#[derive(Clone, Copy, Debug)]
pub struct HeadServer;

impl self::Service for HeadServer {

    type GetFut = Ready<String>;
    type PutFut = Ready<String>;

    fn put(self, _: context::Context, s: Vec<u8>) -> Self::PutFut {
        future::ready(format!("PUT, {}!", str::from_utf8(&s).unwrap()))
    }

    fn get(self, _: context::Context, x: i32, y: i32) -> Self::GetFut {
        future::ready(format!("GET!"))
    }
}

