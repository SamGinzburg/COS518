use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::str;

service! {
    // RPC's for the intermediate server
    //
    //  ----------------------      ---------------------
    //  | Intermediate Server | ->  | Dead Drop Server  |
    //  ----------------------      ---------------------
    //                    AND
    //  ------------------------      ---------------------
    //  | Intermediate Server  | <--  | Dead Drop Server  |
    //  ------------------------      ---------------------
    //
    // TODO
    rpc put(message: Vec<u8>) -> String;
    rpc get(x: i32, y: i32) -> String;
}

#[derive(Clone, Copy, Debug)]
pub struct DeadDropServer;

impl self::Service for DeadDropServer {
    type GetFut = Ready<String>;
    type PutFut = Ready<String>;

    fn put(self, _: context::Context, s: Vec<u8>) -> Self::PutFut {
        future::ready(format!("PUT, {}!", str::from_utf8(&s).unwrap()))
    }

    fn get(self, _: context::Context, x: i32, y: i32) -> Self::GetFut {
        future::ready(format!("GET!"))
    }
}

