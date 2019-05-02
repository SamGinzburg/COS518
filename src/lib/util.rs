use crate::onion;
use crate::message;
use crate::permute::Permutation;
use rand::distributions::Distribution;
use crate::rand::Rng;

pub struct Settings<D : Distribution<u32>> {
    pub other_pks: Vec<onion::PublicKey>,
    pub sk: onion::PrivateKey,
    pub noise: D,
}

pub struct State {
    keys: Vec<onion::DerivedKey>,
    permutation: Permutation,
    n: usize,
}

// straight-forward implementation; could be optimized
pub fn forward<D>(input : Vec<onion::Message>, settings : &Settings<D>)-> (State, Vec<onion::Message>)
where D : Distribution<u32> {
    let mut rng = rand::thread_rng();
    let n = input.len();

    // unwrap, decrypt, and store keys
    let mut keys : Vec<onion::PublicKey> = Vec::with_capacity(n);
    let mut inners : Vec<onion::Message> = Vec::with_capacity(n);

    for wrapped in input {
        let (pk, cipher) = onion::unwrap(&wrapped);
        let dk = onion::derive(&settings.sk, &pk);
        let inner = onion::decrypt(&dk, cipher, onion::EncryptionPurpose::Forward);

        keys.push(dk);
        inners.push(inner);
    }

    // add noise
    let n1 = settings.noise.sample(&mut rng);
    let n2 = settings.noise.sample(&mut rng) / 2;

    let adding = (n1 + 2*n2) as usize;
    inners.reserve(adding);
    let m = n + adding;

    for _i in 0..n1 {
        let m = message::blank(&message::Deaddrop::sample());
        let (_dks, wrapped) = onion::forward_onion_encrypt(&settings.other_pks, m);
        inners.push(wrapped);
    }

    for _i in 0..n2 {
        for _j in 0..2 {
            let m = message::blank(&message::Deaddrop::sample());
            let (_dks, wrapped) = onion::forward_onion_encrypt(&settings.other_pks, m);
            inners.push(wrapped);
        }
    }
    
    // permute
    let permutation = Permutation::sample(m);
    let output : Vec<onion::Message> = permutation.apply(inners);

    (State{ keys, permutation, n }, output)
}

pub fn backward(state : State, input : Vec<onion::Message>) -> Vec<onion::Message> {
    // unpermute
    let unpermuted = state.permutation.apply_inverse(input);

    // re-encrypt
    let mut ciphers : Vec<onion::Message> = Vec::with_capacity(state.n);
    for (m, dk) in unpermuted.iter().zip(state.keys.iter()) {
        // TODO: avoid cloning m
        let c = onion::encrypt(dk, m.to_vec(), onion::EncryptionPurpose::Backward);
        ciphers.push(c);
    }

    ciphers
}

fn deaddrop(input : Vec<onion::Message>) -> Vec<onion::Message> {
    let n = input.len();

    let mut unpacked : Vec<(message::PlaintextMessage, u32)>
        = Vec::with_capacity(n);
    for w in input {
        let (m, d) = message::unpack(w);
        unpacked.push((m, d.location()));
    }


    let mut order = Permutation::from_sort(
        &mut unpacked, |(_m1, d1), (_m2, d2)| d1.partial_cmp(d2).unwrap());

    let mut res_messages : Vec<onion::Message> = Vec::with_capacity(n);
    let mut iter = unpacked.iter();
    let mut prev_d = None;
    let mut current = iter.next();

    while let Some((m,d)) = current {

        // deal with collisions
        if let Some(dd) = prev_d {
            if d == dd {
                // TODO: print some context. Need abort round instead?
                eprintln!("Deaddrop collision. Some messages may not be delivered.");
            }
        }
        prev_d = Some(d);

        // check next
        current = iter.next();
        match current {
            Some((m_other, d_other)) if d == d_other => {
                // swapping order if matched
                res_messages.push(m_other.to_vec());
                res_messages.push(m.to_vec());
                // and advance
                current = iter.next();
            },
            _ => {
                // loner
                res_messages.push(m.to_vec());
            },
        }
    }
    
    order.apply_inverse(res_messages)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deaddrop_switches() {
        let d_shared = message::Deaddrop::from_bytes(&[1,1,1,1]);
        let d_loner = message::Deaddrop::from_bytes(&[2,2,2,2]);
        let m1 = [1; message::CONTENT_SIZE];
        let m2 = [2; message::CONTENT_SIZE];
        let m3 = [3; message::CONTENT_SIZE];

        let input = vec![
            message::pack(&m1, &d_shared),
            message::pack(&m2, &d_loner),
            message::pack(&m3, &d_shared),
        ];

        assert_eq!(deaddrop(input), vec![m3, m2, m1]);
    }
}
