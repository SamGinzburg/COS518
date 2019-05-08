extern crate sharedlib;
use crate::sharedlib::{keys, onion};

fn main() {
    keys::makedirs().expect("Failed to make dirs");

    for i in 0..sharedlib::NUM_CLIENTS {
        keys::put(keys::PartyType::Client.with_id(i), onion::keygen())
            .expect("Failed to write client key");
    }

    for i in 0..sharedlib::NUM_SERVERS {
        keys::put(keys::PartyType::Server.with_id(i), onion::keygen())
            .expect("Failed to write server key");
    }
}
