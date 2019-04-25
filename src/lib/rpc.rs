use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
//use serde::{Deserialize, Serialize};

service! {
    // RPC's for the head server
    rpc put(message: String) -> String;
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

    fn put(self, _: context::Context, s: String) -> Self::PutFut {
        future::ready(format!("PUT, {}!", s))
    }

    fn get(self, _: context::Context, x: i32, y: i32) -> Self::GetFut {
        future::ready(format!("GET!"))
    }
}

