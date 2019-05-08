use crate::message;
use crate::onion;
use crate::permute::Permutation;
use crate::rand::distributions::Distribution;

use std::collections::HashMap;

pub struct Settings<D: Distribution<u32>> {
    pub other_pks: Vec<onion::PublicKey>,
    pub sk: onion::PrivateKey,
    pub noise: D,
}

#[derive(Debug)]
pub struct State {
    keys: Vec<onion::DerivedKey>,
    permutation: Permutation,
    n: usize,
}

// straight-forward implementation; could be optimized
pub fn forward<D>(
    input: Vec<onion::Message>,
    settings: &Settings<D>,
) -> (State, Vec<onion::Message>)
where
    D: Distribution<u32>,
{
    let mut rng = rand::thread_rng();
    let n = input.len();

    // unwrap, decrypt, and store keys
    let mut keys: Vec<onion::PublicKey> = Vec::with_capacity(n);
    let mut inners: Vec<onion::Message> = Vec::with_capacity(n);

    for wrapped in input {
        let (pk, cipher) = message::unwrap(&wrapped);
        let dk = onion::derive(&settings.sk, &pk);
        let inner = onion::decrypt(&dk, cipher, onion::EncryptionPurpose::Forward);

        keys.push(dk);
        inners.push(inner);
    }

    // add noise
    let n1 = settings.noise.sample(&mut rng);
    let n2 = settings.noise.sample(&mut rng) / 2;

    let adding = (n1 + 2 * n2) as usize;
    inners.reserve(adding);
    let m = n + adding;

    for _i in 0..n1 {
        let m = message::blank(&message::Deaddrop::sample());
        let (_dks, wrapped) = message::forward_onion_encrypt(&settings.other_pks, m);
        inners.push(wrapped);
    }

    for _i in 0..n2 {
        for _j in 0..2 {
            let m = message::blank(&message::Deaddrop::sample());
            let (_dks, wrapped) = message::forward_onion_encrypt(&settings.other_pks, m);
            inners.push(wrapped);
        }
    }

    // permute
    let permutation = Permutation::sample(m);
    let output: Vec<onion::Message> = permutation.apply(inners);

    (
        State {
            keys,
            permutation,
            n,
        },
        output,
    )
}

pub fn backward(state: State, input: Vec<onion::Message>) -> Vec<onion::Message> {
    // unpermute
    let mut unpermuted = state.permutation.inverse().apply(input);

    // re-encrypt
    let mut ciphers: Vec<onion::Message> = Vec::with_capacity(state.n);
    for (m, dk) in unpermuted.drain(..).zip(state.keys.iter()) {
        let c = onion::encrypt(dk, m, onion::EncryptionPurpose::Backward);
        ciphers.push(c);
    }

    ciphers
}

enum DeaddropState {
    Once(usize),
    Twice,
}

pub fn deaddrop(mut input: Vec<onion::Message>) -> Vec<onion::Message> {
    const HASH_MARGIN: usize = 1; // tune up as needed to prevent map reallocation

    let n = input.len();
    let mut map: HashMap<u32, DeaddropState> = HashMap::with_capacity(HASH_MARGIN * n);
    let mut output: Vec<onion::Message> = Vec::with_capacity(n);

    for (i, w) in input.drain(0..).enumerate() {
        let (m, d) = message::unpack(w);
        let dl = d.location();
        output.push(m);

        match map.remove(&dl) {
            Some(DeaddropState::Twice) => {
                eprintln!(
                    "Deaddrop collision in {}. Some messages may not be delivered.",
                    dl
                );
            }
            Some(DeaddropState::Once(j)) => {
                let mm = output.swap_remove(j);
                output.push(mm);
                map.insert(dl, DeaddropState::Twice);
            }
            None => {
                map.insert(dl, DeaddropState::Once(i));
            }
        }
    }

    output
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deaddrop_switches() {
        let d_loner = message::Deaddrop::from_bytes(&[1, 1, 1, 1]);
        let d_shared = message::Deaddrop::from_bytes(&[2, 2, 2, 2]);
        let m1 = vec![1; *message::CONTENT_SIZE];
        let m2 = vec![2; *message::CONTENT_SIZE];
        let m3 = vec![3; *message::CONTENT_SIZE];

        let input = vec![
            message::pack(&m1, &d_shared),
            message::pack(&m2, &d_shared),
            message::pack(&m3, &d_loner),
        ];

        assert_eq!(deaddrop(input), vec![m2, m1, m3]);
    }
}
