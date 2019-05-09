use crate::byteorder::{BigEndian, ByteOrder};
use crate::{message, onion};

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
    let pk_bytes = BigEndian::read_u32(&pk[..4]);
    let p = onion::EncryptionPurpose::FromBytes(round ^ pk_bytes);
    let e = onion::encrypt(&dk, m, p);

    // pack with deaddrop
    let mut round_bytes = [0; 4];
    BigEndian::write_u32(&mut round_bytes, round);
    let drop = message::Deaddrop::new(dk, &round_bytes);
    let w = message::pack(&e, &drop);

    // onion encrypt
    message::forward_onion_encrypt(server_pks, w)
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
) -> Result<Vec<u8>, ()> {
    // onion decrypt
    let m = message::backward_onion_decrypt(&server_dks, c)?;

    // decrypt using Alice/Bob shared key
    let pk_bytes = BigEndian::read_u32(&pk[..4]);
    let p = onion::EncryptionPurpose::FromBytes(round ^ pk_bytes);
    onion::decrypt(&dk, m, p)
}
