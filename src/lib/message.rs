use crate::rand::Rng;
use crate::onion;

pub const CONTENT_SIZE : usize = 16;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Deaddrop {
    location : u32,
}

impl Deaddrop {
    pub fn new(dk : &onion::DerivedKey) -> Deaddrop {
        Deaddrop::from_bytes(&dk[..4])
    }

    pub fn from_bytes(bytes : &[u8]) -> Deaddrop {
        // TODO: check endianness
        let location =
            ((bytes[0] as u32) << 24) +
            ((bytes[1] as u32) << 16) +
            ((bytes[2] as u32) <<  8) +
            ((bytes[3] as u32) <<  0);
        Deaddrop { location }
    }

    pub fn sample() -> Deaddrop {
        let location = rand::thread_rng().gen();
        Deaddrop { location }
    }

    pub fn bytes(&self) -> [u8; 4] {
        [
            ((self.location >> 24) & 0xff) as u8,
            ((self.location >> 16) & 0xff) as u8,
            ((self.location >> 8) & 0xff) as u8,
            ((self.location >> 0) & 0xff) as u8,
        ]
    }

    pub fn location(&self) -> u32 {
        self.location
    }
}

pub type PlaintextMessage = [u8; CONTENT_SIZE];

pub fn blank(d : &Deaddrop) -> onion::Message {
    pack(&[0; CONTENT_SIZE], d)
}

pub fn pack(m : &PlaintextMessage, d : &Deaddrop) -> onion::Message {
    let mut p = Vec::with_capacity(4 + CONTENT_SIZE);
    p.extend(m);
    p.extend(&d.bytes());
    p
}

pub fn unpack(w : onion::Message) -> (PlaintextMessage, Deaddrop) {
    let mut m : [u8; CONTENT_SIZE] = Default::default();
    m.copy_from_slice(&w[..CONTENT_SIZE]);
    let d = Deaddrop::from_bytes(&w[CONTENT_SIZE..]);
    (m, d)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::onion;

    #[test]
    fn from_bytes_correct() {
        let b : [u8; 4] = [1, 2, 3, 4];
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
        let m = [123; 16];
        let d = Deaddrop::sample();
        let p = pack(&m, &d);
        let (mm, dd) = unpack(p.clone());

        assert_eq!(m, mm);
        assert_eq!(d, dd);
    }
}
