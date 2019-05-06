use crate::{message, onion};

/// For Alice to wrap a message to send to Bob over servers s1...sn.
/// Put:
///  round : the round number
///  m : of length message::RAW_SIZE
///  dk = onion::derive(&sk_alice, &pk_bob)
///  server_pks : pks of s1...sn
pub fn wrap(
    round : u32,
    mut m : Vec<u8>,
    dk : &onion::DerivedKey,
    server_pks : &Vec<onion::PublicKey>,
) -> (Vec<onion::DerivedKey>, onion::Message) {
    // resize
    m.resize(message::RAW_SIZE, 0);

    // encrypt for Bob
    // TODO: Bob and Alice must encrypt with different salts
    let e = onion::encrypt(&dk, m, onion::EncryptionPurpose::InnerForRound(round));

    // pack with deaddrop
    let drop = message::Deaddrop::new(dk);
    let w = message::pack(&e, &drop);

    // onion encrypt
    onion::forward_onion_encrypt(server_pks, w)
}

/// For Alice to unwrap her message received from Bob via servers
pub fn unwrap(
    round : u32,
    mut c : onion::Message,
    dk : &onion::DerivedKey,
    server_dks : Vec<onion::DerivedKey>,
) -> Vec<u8> {
    // onion decrypt
    let m = onion::backward_onion_decrypt(&server_dks, c);

    // decrypt using Alice/Bob shared key
    onion::decrypt(&dk, m, onion::EncryptionPurpose::InnerForRound(round))
}
