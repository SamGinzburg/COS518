use crate::rand::Rng;

// TODO:
// these are all dummy operations
// need replace with crypto library

pub type PrivateKey = i32;
pub type PublicKey = i32;
pub type DerivedKey = i32;
pub type Message = String;

pub fn blank_message(deaddrop : u32) -> Message {
    // TODO: make length match real message
    let mut m = String::new();
    m.push_str(&deaddrop.to_string());
    m.push_str("          ");
    m.to_string()
}

pub fn keygen<T: Rng>(rng : &mut T) -> (PrivateKey, PublicKey) {
    let x = rng.gen();
    let y = 100 * x;
    (y, x)
}

pub fn wrap(k : &PublicKey, m : &Message) -> Message {
    let mut w = String::new();
    w.push_str(&k.to_string());
    w.push('|');
    w.push_str(&m);
    w.to_string()
}

pub fn unwrap(w : &Message) -> (PublicKey, Message) {
    let mut split = w.splitn(2, '|');
    let k = split.next().unwrap().parse::<i32>().unwrap();
    let m = split.next().unwrap();
    (k,m.to_string())
}

pub fn derive(k1 : &PrivateKey, k2: &PublicKey) -> DerivedKey {
    (k1 * k2)
}

pub fn encrypt(k : &DerivedKey, m : &Message) -> Message {
    m.clone()
}

pub fn decrypt(k : &DerivedKey, c : &Message) -> Message {
    c.clone()
}


// this method actually done once lower-level ones use crypto
pub fn rec_wrap<T: Rng>(pks : &Vec<PublicKey>, m : &Message, rng : &mut T)
                        -> (Vec<DerivedKey>, Message) {

    let mut m = m.clone();
    let mut dks = Vec::with_capacity(pks.len());

    for pk_server in pks.iter().rev() {
        let (sk, pk) = keygen(rng);
        let dk = derive(&sk, &pk_server);
        let c = encrypt(&dk, &m);
        m = wrap(&pk, &c);
        dks.push(dk);
    }

    (dks, m)
}


#[cfg(test)]
mod text {
    use super::*;
    use crate::rand::rngs::mock::StepRng;

    #[test]
    fn keygen_randomized() {
        let mut rng = StepRng::new(0,1);
        let (sk1, pk1) = keygen(&mut rng);
        let (sk2, pk2) = keygen(&mut rng);
        
        assert_ne!(sk1, sk2);
        assert_ne!(pk1, pk2);
    }
    
    #[test]
    fn wrap_invertible() {
        let mut rng = StepRng::new(0,1);
        let (sk, pk) = keygen(&mut rng);
        let m = "Hello, world!".to_string();
        let w = wrap(&pk, &m);
        let (pk_uw, m_uw) = unwrap(&w);

        assert_eq!(pk_uw, pk);
        assert_eq!(m, m_uw);
    }

    #[test]
    fn derive_commutes() {
        let mut rng = StepRng::new(0,1);
        let (sk1, pk1) = keygen(&mut rng);
        let (sk2, pk2) = keygen(&mut rng);
        let d1 = derive(&sk1, &pk2);
        let d2 = derive(&sk2, &pk1);

        assert_eq!(d1, d2);
    }

    #[test]
    fn encrypt_invertible() {
        let mut rng = StepRng::new(0,1);
        let (sk1, pk1) = keygen(&mut rng);
        let (sk2, pk2) = keygen(&mut rng);
        let d = derive(&sk1, &pk2);
        
        let m = "Hello, world!".to_string();
        let c = encrypt(&d, &m);
        let m_dc = decrypt(&d, &c);

        assert_eq!(m, m_dc);
    }

    #[test]
    fn integration() {
        let mut rng = StepRng::new(0,1);
        let (sk_server, pk_server) = keygen(&mut rng);
        let (sk_client, pk_client) = keygen(&mut rng);

        // client
        let m = "Hello, server!".to_string();
        let d_client = derive(&sk_client, &pk_server);
        let c = encrypt(&d_client, &m);
        let w = wrap(&pk_client, &c);

        // server
        let (pk_unwrapped, c_unwrapped) = unwrap(&w);
        let d_server = derive(&sk_server, &pk_unwrapped);
        let m_server = decrypt(&d_server, &c_unwrapped);
        assert_eq!(m, m_server);
        let r = "Hello, client!".to_string();
        let c_server = encrypt(&d_server, &r);

        // client
        let r_client = decrypt(&d_client, &c_server);
        assert_eq!(r, r_client);
    }

    #[test]
    fn rec_wrap_inverts() {
        let mut rng = StepRng::new(0,1);
        let (sk1, pk1) = keygen(&mut rng);
        let (sk2, pk2) = keygen(&mut rng);

        let m = "Hello, onions!".to_string();
        let (dks, w) = rec_wrap(&vec!(pk1, pk2), &m, &mut rng);

        // server 1 unwrap decrypt
        let (pku, c) = unwrap(&w);
        let d = derive(&sk1, &pku);
        let w = decrypt(&d, &c);

        // server 2 unwrap decrypt
        let (pku, c) = unwrap(&w);
        let d = derive(&sk2, &pku);
        let w = decrypt(&d, &c);

        assert_eq!(m, w);
    }
}
