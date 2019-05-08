use crate::byteorder::{BigEndian, ByteOrder};
use crate::ring::{aead, agreement, digest, hkdf, rand};
use std::sync::Mutex;

pub type PrivateKey = Vec<u8>; // bytes of sk
pub type PublicKey = Vec<u8>; // bytes of pk
pub type KeyPair = (PrivateKey, PublicKey);

pub type DerivedKey = Vec<u8>;
pub type Message = Vec<u8>;

pub enum EncryptionPurpose {
    Forward,
    Backward,
    FromBytes(u32),
}

fn bytes(v: u32) -> [u8; aead::NONCE_LEN] {
    let mut b = [0; aead::NONCE_LEN];
    BigEndian::write_u32(&mut b, v);
    b
}

impl Into<[u8; aead::NONCE_LEN]> for EncryptionPurpose {
    fn into(self) -> [u8; aead::NONCE_LEN] {
        bytes(match self {
            EncryptionPurpose::Forward => 0,
            EncryptionPurpose::Backward => 1,
            EncryptionPurpose::FromBytes(b) => b,
        })
    }
}

static AGREEMENT: &agreement::Algorithm = &agreement::X25519;
static AEAD: &aead::Algorithm = &aead::AES_256_GCM;
static DIGEST: &digest::Algorithm = &digest::SHA256;

lazy_static! {
    static ref RNG: Mutex<rand::SystemRandom> = Mutex::new(rand::SystemRandom::new());
    pub static ref PK_LEN: usize = {
        let (_sk, pk) = keygen();
        pk.len()
    };
    pub static ref TAG_LEN: usize = { AEAD.tag_len() };
}

macro_rules! rng {
    () => {
        &*(RNG.lock().expect("Could not obtain Ring RNG."))
    };
}

// TODO: pass errors up!

pub fn keygen() -> (PrivateKey, PublicKey) {
    let keys =
        agreement::EphemeralPrivateKey::generate(AGREEMENT, rng!()).expect("Key agreement failed");

    let pk = keys.compute_public_key().unwrap().as_ref().to_vec();
    let sk = keys.as_ref().to_vec();

    (sk, pk)
}

pub fn derive(k1: &PrivateKey, k2: &PublicKey) -> DerivedKey {
    lazy_static! {
        // TODO (optional): use a non-empty salt value
        static ref SALT : hkdf::Salt = hkdf::Salt::new(DIGEST, &[]);
    }

    // key bytes to objects
    let upk = agreement::UnparsedPublicKey::new(AGREEMENT, k2);
    let usk = agreement::EphemeralPrivateKey::new(AGREEMENT, k1).unwrap();

    // secret point from key exchange
    let secret =
        agreement::agree_ephemeral(&usk, &upk, ring::error::Unspecified, |s| Ok(s.to_vec()))
            .expect("Key agreement failed");

    // process into well-distributed AEAD key
    let mut out: Vec<u8> = vec![0; AEAD.key_len()];
    SALT.extract(&secret)
        .expand(&[])
        .fill(&mut out)
        .expect("Could not extract and expand secret");

    out
}

pub fn encrypt(k: &DerivedKey, m: Message, p: EncryptionPurpose) -> Message {
    let sealing_key = aead::SealingKey::new(AEAD, k).expect("Cannot encrypt using derived key.");

    let nonce = aead::Nonce::assume_unique_for_key(p.into());

    let aad = aead::Aad::empty();

    let mut in_out: Vec<u8> = Vec::with_capacity(m.len() + AEAD.tag_len());
    in_out.extend(m);
    in_out.extend(vec![0; AEAD.tag_len()]);

    aead::seal_in_place(&sealing_key, nonce, aad, &mut in_out, AEAD.tag_len())
        .expect("Encryption failed");

    in_out
}

pub fn decrypt(k: &DerivedKey, mut c: Message, p: EncryptionPurpose) -> Result<Message, ()> {
    let opening_key = aead::OpeningKey::new(AEAD, k).expect("Cannot decrypt using derived key.");

    let nonce = aead::Nonce::assume_unique_for_key(p.into());

    let aad = aead::Aad::empty();

    match aead::open_in_place(&opening_key, nonce, aad, 0, &mut c) {
        Err(_) => Err(()),
        Ok(result) => Ok(result.to_vec()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn keygen_randomized() {
        let (_sk, pk1) = keygen();
        let (_sk, pk2) = keygen();

        assert_ne!(pk1, pk2);
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
        let m_dc = decrypt(&d, c, EncryptionPurpose::Forward).unwrap();
        assert_eq!(m, m_dc);
    }

    #[test]
    fn decrypt_can_fail() {
        let (sk1, pk1) = keygen();
        let (_sk2, pk2) = keygen();
        let d1 = derive(&sk1, &pk2);
        let d2 = derive(&sk1, &pk1);

        let m = "Hello, world!".as_bytes().to_vec();
        let c = encrypt(&d1, m, EncryptionPurpose::Forward);
        let dc = decrypt(&d2, c, EncryptionPurpose::Forward);
        assert_eq!(dc, Err(()));
    }
}
