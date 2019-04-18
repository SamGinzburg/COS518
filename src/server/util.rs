use sharedlib::onion;
use crate::permute::Permutation;
use rand::distributions::Distribution;
use crate::rand::Rng;

struct Settings<D : Distribution<u32>> {
    other_pks: Vec<onion::PublicKey>,
    sk: onion::PrivateKey,
    noise: D,
}

struct State {
    keys: Vec<onion::DerivedKey>,
    permutation: Permutation,
    n: usize,
}

// straight-forward implementation; could be optimized
fn forward<D>(input : Vec<onion::Message>, settings : &Settings<D>)-> (State, Vec<onion::Message>)
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
        let deaddrop = rng.gen();
        let m = onion::blank_message(deaddrop);
        let (_dks, wrapped) = onion::forward_onion_encrypt(&settings.other_pks, m);
        inners.push(wrapped);
    }

    for _i in 0..n2 {
        let deaddrop = rng.gen();
        for _j in 0..2 {
            let m = onion::blank_message(deaddrop);
            let (_dks, wrapped) = onion::forward_onion_encrypt(&settings.other_pks, m);
            inners.push(wrapped);
        }
    }
    
    // permute
    let permutation = Permutation::sample(m);
    let output : Vec<onion::Message> = permutation.apply(inners);

    (State{ keys, permutation, n }, output)
}

fn backward(state : State, input : Vec<onion::Message>) -> Vec<onion::Message> {
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
