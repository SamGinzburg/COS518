use crate::ring::{agreement, aead, digest, hkdf, hmac, rand};
use std::sync::Mutex;

pub type PrivateKey = agreement::EphemeralPrivateKey;
pub type PublicKey = Vec<u8>; // bytes of pk
pub type DerivedKey = Vec<u8>;
pub type Message = Vec<u8>;

pub enum EncryptionPurpose {
    Forward = 1,
    Backward = 2,
}

static AGREEMENT : &agreement::Algorithm = &agreement::X25519;
static AEAD : &aead::Algorithm = &aead::AES_256_GCM;
static DIGEST : &digest::Algorithm = &digest::SHA256;

lazy_static! {
    static ref RNG : Mutex<rand::SystemRandom> =
        Mutex::new(rand::SystemRandom::new());

    static ref PK_LEN : usize = {
        let (_sk, pk) = keygen();
        pk.len()
    };
}

macro_rules! rng {
    () => {&*(RNG.lock().expect("Could not obtain Ring RNG."))}
}

// TODO: pass errors up!

pub fn keygen() -> (PrivateKey, PublicKey) {
    let sk = agreement::EphemeralPrivateKey::generate(AGREEMENT, rng!())
        .expect("Key agreement failed");
    let pk = sk.compute_public_key().unwrap().as_ref().to_vec();

    (sk, pk)
}

pub fn wrap(k : &PublicKey, m : &Message) -> Message {
    let mut w = Vec::with_capacity(*PK_LEN + m.len());
    w.extend(k);
    w.extend(m);
    w
}

pub fn unwrap(w : &Message) -> (PublicKey, Message) {
    (w[..*PK_LEN].to_vec(), w[*PK_LEN..].to_vec())
}

pub fn derive(k1 : &PrivateKey, k2: &PublicKey) -> DerivedKey {
    lazy_static! {
        // TODO (optional): use a non-empty salt value
        static ref SALT : hkdf::Salt = hkdf::Salt::new(DIGEST, &[]);
    }

    let upk = agreement::UnparsedPublicKey::new(AGREEMENT, k2);

    // secret point from key exchange
    let secret = agreement::agree_ephemeral(
        k1, &upk, ring::error::Unspecified, |s| {
            Ok(s.to_vec())
        }
    ).expect("Key agreement failed");

    // process into well-distributed AEAD key
    let mut out : Vec<u8> = vec![0; AEAD.key_len()];
    SALT.extract(&secret).expand(&[]).fill(&mut out)
        .expect("Could not extract and expand secret");

    out
}

pub fn encrypt(k : &DerivedKey, m : Message, p : EncryptionPurpose) -> Message {
    let sealing_key = aead::SealingKey::new(AEAD, k)
        .expect("Cannot encrypt using derived key.");

    let nonce = aead::Nonce::assume_unique_for_key([p as u8; aead::NONCE_LEN]);

    let aad = aead::Aad::empty();

    let mut in_out : Vec<u8> = Vec::with_capacity(m.len() + AEAD.tag_len());
    in_out.extend(m);
    in_out.extend(vec![0; AEAD.tag_len()]);

    aead::seal_in_place(&sealing_key, nonce, aad, &mut in_out, AEAD.tag_len())
        .expect("Encryption failed");

    in_out
}

pub fn decrypt(k : &DerivedKey, mut c : Message, p : EncryptionPurpose) -> Message {
    let opening_key = aead::OpeningKey::new(AEAD, k)
        .expect("Cannot decrypt using derived key.");

    let nonce = aead::Nonce::assume_unique_for_key([p as u8; aead::NONCE_LEN]);

    let aad = aead::Aad::empty();

    aead::open_in_place(&opening_key, nonce, aad, 0, &mut c)
        .expect("Encryption failed")
        .to_vec()
}


pub fn forward_onion_encrypt(pks : &Vec<PublicKey>, mut m : Message) -> (Vec<DerivedKey>, Message) {
    let mut dks = Vec::with_capacity(pks.len());

    for pk_server in pks.iter().rev() {
        let (sk, pk) = keygen();
        let dk = derive(&sk, &pk_server);
        let c = encrypt(&dk, m, EncryptionPurpose::Forward);
        m = wrap(&pk, &c);
        dks.push(dk);
    }

    dks.reverse();

    (dks, m)
}

pub fn backward_onion_decrypt(dks : &Vec<DerivedKey>, mut c : Message) -> Message {
    for dk in dks.iter() {
        println!("{:?}", c);
        c = decrypt(&dk, c, EncryptionPurpose::Backward);
    }
    println!("{:?}", c);
    c
}

#[cfg(test)]
mod text {
    use super::*;

    #[test]
    fn keygen_randomized() {
        let (_sk, pk1) = keygen();
        let (_sk, pk2) = keygen();

        assert_ne!(pk1, pk2);
    }

    #[test]
    fn wrap_invertible() {
        let (_sk, pk) = keygen();
        let m = "Hello, world!".as_bytes().to_vec();
        let w = wrap(&pk, &m);
        let (pk_uw, m_uw) = unwrap(&w);

        assert_eq!(pk_uw, pk);
        assert_eq!(m, m_uw);
    }

    #[test]
    fn derive_commutes() {
        let (sk1, pk1) = keygen();
        let (sk2, pk2) = keygen();
        let d1 = derive(&sk1, &pk2);
        let d2 = derive(&sk2, &pk1);

        assert_eq!(d1, d2);
    }

    #[test]
    fn encrypt_invertible() {
        let (sk1, _pk1) = keygen();
        let (_sk2, pk2) = keygen();
        let d = derive(&sk1, &pk2);
        
        let m = "Hello, world!".as_bytes().to_vec();
        let c = encrypt(&d, m.clone(), EncryptionPurpose::Forward);
        let m_dc = decrypt(&d, c, EncryptionPurpose::Forward);
        assert_eq!(m, m_dc);
    }

    #[test]
    fn integration() {
        let (sk_server, pk_server) = keygen();
        let (sk_client, pk_client) = keygen();

        // client
        let m = "Hello, server!".as_bytes().to_vec();
        let d_client = derive(&sk_client, &pk_server);
        let c = encrypt(&d_client, m.clone(), EncryptionPurpose::Forward);
        let w = wrap(&pk_client, &c);

        // server
        let (pk_unwrapped, c_unwrapped) = unwrap(&w);
        let d_server = derive(&sk_server, &pk_unwrapped);
        let m_server = decrypt(&d_server, c_unwrapped, EncryptionPurpose::Forward);
        assert_eq!(m, m_server);
        let r = "Hello, client!".as_bytes().to_vec();
        let c_server = encrypt(&d_server, r.clone(), EncryptionPurpose::Backward);

        // client
        let r_client = decrypt(&d_client, c_server, EncryptionPurpose::Backward);
        assert_eq!(r, r_client);
    }

    #[test]
    fn test_onion() {
        let (sk1, pk1) = keygen();
        let (sk2, pk2) = keygen();

        let m = "Hello, onions!".as_bytes().to_vec();

        // client encrypts
        let (dks, w) = forward_onion_encrypt(&vec!(pk1, pk2), m.clone());

        // server 1 unwrap decrypt
        let (pku, c) = unwrap(&w);
        let d1 = derive(&sk1, &pku);
        let w = decrypt(&d1, c, EncryptionPurpose::Forward);

        // server 2 unwrap decrypt
        let (pku, c) = unwrap(&w);
        let d2 = derive(&sk2, &pku);
        let w = decrypt(&d2, c, EncryptionPurpose::Forward);

        assert_eq!(m, w);

        let m = "Hello, client!".as_bytes().to_vec();

        // server 2 re-encrypts
        let c = encrypt(&d2, m.clone(), EncryptionPurpose::Backward);

        // server 1 re-encrypts
        let c = encrypt(&d1, c, EncryptionPurpose::Backward);

        // client decrypts
        let n = backward_onion_decrypt(&dks, c);

        assert_eq!(m, n);
    }
}
