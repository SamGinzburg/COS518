use crate::{message, onion};

fn from_bytes(bytes: &Vec<u8>) -> u32 {
    (((bytes[0] as u32) << 24)
        + ((bytes[1] as u32) << 16)
        + ((bytes[2] as u32) << 8)
        + ((bytes[3] as u32) << 0))
}

/// For Alice to wrap a message to send to Bob over servers s1...sn.
/// Put:
///  round : the round number
///  m : of length message::RAW_SIZE
///  pk = &pk_bob
///  dk = onion::derive(&sk_alice, &pk_bob)
///  server_pks : pks of s1...sn
pub fn wrap(
    round: u32,
    mut m: Vec<u8>,
    pk: &onion::PublicKey,
    dk: &onion::DerivedKey,
    server_pks: &Vec<onion::PublicKey>,
) -> (Vec<onion::DerivedKey>, onion::Message) {
    // resize
    m.resize(message::RAW_SIZE, 0);

    // encrypt for Bob
    let p = onion::EncryptionPurpose::FromBytes(round ^ from_bytes(pk));
    let e = onion::encrypt(&dk, m, p);

    // pack with deaddrop
    let drop = message::Deaddrop::new(dk);
    let w = message::pack(&e, &drop);

    // onion encrypt
    onion::forward_onion_encrypt(server_pks, w)
}

/// For Alice to unwrap her message received from Bob via servers
/// Put:
///  round : the round number
///  c : response from server
///  pk = &pk_alice
///  dk = onion::derive(&sk_alice, &pk_bob)
///  server_dks : output from wrap
pub fn unwrap(
    round: u32,
    c: onion::Message,
    pk: &onion::PublicKey,
    dk: &onion::DerivedKey,
    server_dks: Vec<onion::DerivedKey>,
) -> Vec<u8> {
    // onion decrypt
    let m = onion::backward_onion_decrypt(&server_dks, c);

    // decrypt using Alice/Bob shared key
    let p = onion::EncryptionPurpose::FromBytes(round ^ from_bytes(pk));
    onion::decrypt(&dk, m, p)
}
