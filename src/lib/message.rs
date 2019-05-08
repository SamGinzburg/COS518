use crate::byteorder::{BigEndian, ByteOrder};
use crate::onion::{self, EncryptionPurpose, DerivedKey, Message, PublicKey};
use crate::rand::Rng;

pub const RAW_SIZE: usize = 256;

lazy_static! {
    pub static ref CONTENT_SIZE: usize = RAW_SIZE + *onion::TAG_LEN;
}

pub fn wrap(k: &PublicKey, m: &Message) -> Message {
    let mut w = Vec::with_capacity(*onion::PK_LEN + m.len());
    w.extend(k);
    w.extend(m);
    w
}

pub fn unwrap(w: &Message) -> (PublicKey, Message) {
    (w[..*onion::PK_LEN].to_vec(), w[*onion::PK_LEN..].to_vec())
}

pub fn forward_onion_encrypt(pks: &Vec<PublicKey>, mut m: Message) -> (Vec<DerivedKey>, Message) {
    let mut dks = Vec::with_capacity(pks.len());

    for pk_server in pks.iter().rev() {
        let (sk, pk) = onion::keygen();
        let dk = onion::derive(&sk, &pk_server);
        let c = onion::encrypt(&dk, m, EncryptionPurpose::Forward);
        m = wrap(&pk, &c);
        dks.push(dk);
    }

    dks.reverse();

    (dks, m)
}

pub fn backward_onion_decrypt(dks: &Vec<DerivedKey>, mut c: Message) -> Message {
    for dk in dks.iter() {
        //println!("Backwards decrypt msg: {:?}", c);
        c = onion::decrypt(&dk, c, EncryptionPurpose::Backward);
    }
    //println!("{:?}", c);
    c
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Deaddrop {
    location: u32,
}

impl Deaddrop {
    pub fn new(dk: &DerivedKey) -> Deaddrop {
        Deaddrop::from_bytes(&dk[..4])
    }

    pub fn from_bytes(bytes: &[u8]) -> Deaddrop {
        let location = BigEndian::read_u32(bytes);
        Deaddrop { location }
    }

    pub fn sample() -> Deaddrop {
        let location = rand::thread_rng().gen();
        Deaddrop { location }
    }

    pub fn location(&self) -> u32 {
        self.location
    }

    fn bytes(&self) -> [u8; 4] {
        let mut buf = [0; 4];
        BigEndian::write_u32(&mut buf, self.location);
        buf
    }
}

pub fn blank(d: &Deaddrop) -> Message {
    pack(&vec![0; *CONTENT_SIZE], d)
}

pub fn pack(m: &Vec<u8>, d: &Deaddrop) -> Message {
    let mut p = Vec::with_capacity(4 + *CONTENT_SIZE);
    p.extend(m);
    p.extend(&d.bytes());
    p
}

pub fn unpack(w: Message) -> (Vec<u8>, Deaddrop) {
    let m = w[..*CONTENT_SIZE].to_vec();
    let d = Deaddrop::from_bytes(&w[*CONTENT_SIZE..]);
    (m, d)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::onion;

    #[test]
    fn wrap_invertible() {
        let (_sk, pk) = onion::keygen();
        let m = "Hello, world!".as_bytes().to_vec();
        let w = wrap(&pk, &m);
        let (pk_uw, m_uw) = unwrap(&w);

        assert_eq!(pk_uw, pk);
        assert_eq!(m, m_uw);
    }

    #[test]
    fn from_bytes_correct() {
        let b: [u8; 4] = [1, 2, 3, 4];
        let drop = Deaddrop::from_bytes(&b);

        assert_eq!(drop.bytes(), b);
    }

    #[test]
    fn from_key_correct() {
        let (sk, _pk) = onion::keygen();
        let (_sk, pk) = onion::keygen();
        let dk = onion::derive(&sk, &pk);
        let drop = Deaddrop::new(&dk);

        assert_eq!(drop.bytes(), dk[..4]);
    }

    #[test]
    fn sample_randomized() {
        let drop1 = Deaddrop::sample();
        let drop2 = Deaddrop::sample();

        assert_ne!(drop1, drop2);
    }

    #[test]
    fn pack_invertible() {
        let m = vec![123; *CONTENT_SIZE];
        let d = Deaddrop::sample();
        let p = pack(&m, &d);
        let (mm, dd) = unpack(p.clone());

        assert_eq!(m, mm);
        assert_eq!(d, dd);
    }

    #[test]
    fn test_onion() {
        let (sk1, pk1) = onion::keygen();
        let (sk2, pk2) = onion::keygen();

        let m = "Hello, onions!".as_bytes().to_vec();

        // client encrypts
        let (dks, w) = forward_onion_encrypt(&vec![pk1, pk2], m.clone());

        // server 1 unwrap decrypt
        let (pku, c) = unwrap(&w);
        let d1 = onion::derive(&sk1, &pku);
        let w = onion::decrypt(&d1, c, EncryptionPurpose::Forward);

        // server 2 unwrap decrypt
        let (pku, c) = unwrap(&w);
        let d2 = onion::derive(&sk2, &pku);
        let w = onion::decrypt(&d2, c, EncryptionPurpose::Forward);

        assert_eq!(m, w);

        let m = "Hello, client!".as_bytes().to_vec();

        // server 2 re-encrypts
        let c = onion::encrypt(&d2, m.clone(), EncryptionPurpose::Backward);

        // server 1 re-encrypts
        let c = onion::encrypt(&d1, c, EncryptionPurpose::Backward);

        // client decrypts
        let n = backward_onion_decrypt(&dks, c);

        assert_eq!(m, n);
    }
}
