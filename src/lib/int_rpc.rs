use tarpc::futures::*;
use tarpc::futures::future::Ready;
use tarpc::{context};
use std::sync::Mutex;

use crate::onion;

lazy_static! {
    // a list of messages, protected by a global lock
    pub static ref MESSAGES: Mutex<Vec<onion::Message>> = Mutex::new(vec![]);
}

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
    rpc SendMessages(v: Vec<onion::Message>) -> bool;
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

    // head server calls this to signify when it is done
    fn EndRound(self, _: context::Context) -> Self::EndRoundFut {
        // when we end a round, clear the MESSAGES vec to prepare for next round
        // also signals us to begin sending messages to deaddrop server
        future::ready(true)
    }

    // both the deaddrop server & head server are calling this RPC
    fn SendMessages(self, _: context::Context, v: Vec<onion::Message>) -> Self::SendMessagesFut {
        let mut m_vec = MESSAGES.lock().unwrap();
        m_vec.extend(v.clone());
        println!("# messages received = {}", m_vec.len());
        future::ready(true)
    }
}

