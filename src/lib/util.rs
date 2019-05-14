use crate::message;
use crate::onion;
use crate::permute::Permutation;
use crate::rand::distributions::Distribution;
use crate::rayon::prelude::*;

use std::collections::HashMap;
use std::time::Instant;

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

pub fn forward<D>(
    input: Vec<onion::Message>,
    settings: &Settings<D>,
) -> (State, Vec<onion::Message>)
where
    D: Distribution<u32> + Sync,
{
    let mut rng = rand::thread_rng();
    let n = input.len();

    // unwrap, decrypt, and store keys
    let mut keys: Vec<onion::DerivedKey> = Vec::with_capacity(n);
    let mut inners: Vec<onion::Message> = Vec::with_capacity(n);

    let now = Instant::now();
    let _unwrapped = input
        .par_iter()
        .map(|wrapped| {
            let (pk, cipher) = message::unwrap(&wrapped);
            let dk = onion::derive(&settings.sk, &pk);
            let inner = match onion::decrypt(&dk, cipher, onion::EncryptionPurpose::Forward) {
                Ok(m) => m,

                // for security, replace bad messages with fakes
                Err(()) => message::blank(&message::Deaddrop::sample()),
            };

            (dk, inner)
        })
        .unzip_into_vecs(&mut keys, &mut inners);
    println!(
        "FORWARD DECRYPT TIME ELAPSED (ms): {}",
        now.elapsed().as_millis()
    );

    // add noise
    let n1 = settings.noise.sample(&mut rng);
    let n2 = settings.noise.sample(&mut rng) / 2;

    let adding = (n1 + 2 * n2) as usize;
    let m = n + adding;

    let now = Instant::now();
    let noise1 = (0..n1).into_par_iter().map(|_| {
        let m = message::blank(&message::Deaddrop::sample());
        let (_dks, wrapped) = message::forward_onion_encrypt(&settings.other_pks, m);
        wrapped
    });

    let noise2 = (0..n2).into_par_iter().flat_map(|_| {
        let r: Vec<onion::Message> = (0..2)
            .into_iter()
            .map(|__| {
                let m = message::blank(&message::Deaddrop::sample());
                let (_dks, wrapped) = message::forward_onion_encrypt(&settings.other_pks, m);
                wrapped
            })
            .collect();
        r
    });

    let all: Vec<onion::Message> = inners.into_par_iter().chain(noise1).chain(noise2).collect();
    println!(
        "FORWARD NOISE ADDITION TIME ELAPSED (ms): {}",
        now.elapsed().as_millis()
    );

    // permute
    let permutation = Permutation::sample(m);
    let output: Vec<onion::Message> = permutation.apply(all);

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
    let unpermuted = state.permutation.inverse().apply(input);

    let now = Instant::now();
    // re-encrypt
    let result = unpermuted
        .into_par_iter()
        .zip(state.keys.par_iter())
        .map(|(m, dk)| onion::encrypt(dk, m, onion::EncryptionPurpose::Backward))
        .collect();

    println!(
        "BACKWARDS Re-encrypt TIME ELAPSED (ms): {}",
        now.elapsed().as_millis()
    );
    result
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
