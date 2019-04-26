use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
//use serde::{Deserialize, Serialize};

service! {
    // RPC's for the head server
    rpc StartNewRound() -> bool;
    rpc EndRound() -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct IntermediateServer;

impl self::Service for IntermediateServer {

    type StartNewRoundFut = Ready<String>;
    type EndRoundFut = Ready<String>;

    fn StartNewRound(self, _: context::Context) -> Self::StartNewRoundFut {
        future::ready(true)
    }

    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
        future::ready(true)
    }
}

