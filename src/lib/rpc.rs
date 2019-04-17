use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};

service! {
    rpc get(name: String) -> String;
    rpc put(x: i32, y: i32) -> i32;
}

#[derive(Clone, Copy, Debug)]
pub struct HeadServer;

impl self::Service for HeadServer {

    type GetFut = Ready<String>;
    type PutFut = Ready<i32>;

    fn get(self, _: context::Context, s: String) -> Self::GetFut {
        future::ready(format!("Hello, {}!", s))
    }

    fn put(self, _: context::Context, x: i32, y: i32) -> Self::PutFut {
        future::ready(x + y)
    }
}

