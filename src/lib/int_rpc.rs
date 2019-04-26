use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
//use serde::{Deserialize, Serialize};

service! {
    // RPC's for the intermediate server
    //
    //  ----------------       ------------------------
    //  | Head Server  |  -->  | Intermediate Server  |
    //  ----------------       ------------------------
    //                    AND
    //  ------------------------      --------------------
    //  | Intermediate Server  | <--  | Deaddrop Server  |
    //  ------------------------      --------------------
    //
    // Head Server ->  Intermediate Server calls
    // starts a round on the next server
    rpc StartNewRound() -> bool;
    // tells the server we are done with the curent round
    rpc EndRound() -> bool;
    // Sends a batch of messages in a round
    rpc SendMessages() -> bool;
}

#[derive(Clone, Copy, Debug)]
pub struct IntermediateServer;

impl self::Service for IntermediateServer {

    type StartNewRoundFut = Ready<bool>;
    type EndRoundFut = Ready<bool>;
    type SendMessagesFut = Ready<bool>;

    fn StartNewRound(self, _: context::Context) -> Self::StartNewRoundFut {
        future::ready(true)
    }

    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
        future::ready(true)
    }

    fn SendMessages(self, _: context::Context) -> Self::SendMessagesFut {
        future::ready(true)
    }
}

