use sharedlib::{client_util, laplace, onion, util};

#[test]
fn crypto_integration_test() {
    // server keys
    let (sk0, pk0) = onion::keygen();
    let (sk1, pk1) = onion::keygen();
    let (sk2, pk2) = onion::keygen();
    let server_pks = vec![pk0, pk1, pk2];

    // client keys
    let (ska, pka) = onion::keygen();
    let (skb, pkb) = onion::keygen();
    let (skc, pkc) = onion::keygen();

    // derived keys
    let dka = onion::derive(&ska, &pkb);
    let dkb = onion::derive(&skb, &pka);
    assert_eq!(dka, dkb);
    let dkc = onion::derive(&skc, &pkc);

    // messages
    let ma = "Hello, Bob!".as_bytes().to_vec();
    let mb = "Hello, Alice!".as_bytes().to_vec();
    let mc = "Hello, Charlie!".as_bytes().to_vec();

    // wrap
    let r = 3;
    let (server_dksa, wa) = client_util::wrap(r, ma, &dka, &server_pks);
    let (server_dksb, wb) = client_util::wrap(r, mb, &dkb, &server_pks);
    let (server_dksc, wc) = client_util::wrap(r, mc, &dkc, &server_pks);
    let in0 = vec![wa, wb, wc];

    // noise
    let laplace = laplace::Laplace::new(1.0, 10.0);
    let noise = laplace::TransformedDistribution::new(laplace, |x| u32::max(0, f64::ceil(x) as u32));

    // server settings
    let s0 = util::Settings {
        other_pks: server_pks[1..].to_vec(),
        sk: sk0,
        noise: noise.clone(),
    };
    let s1 = util::Settings {
        other_pks: server_pks[2..].to_vec(),
        sk: sk1,
        noise: noise.clone(),
    };
    let s2 = util::Settings {
        other_pks: server_pks[3..].to_vec(),
        sk: sk2,
        noise: noise.clone(),
    };

    // forward
    println!("in0 len: {}", in0.len());
    let (s0, in1) = util::forward(in0, &s0);
    println!("in1 len: {}", in1.len());
    let (s1, in2) = util::forward(in1, &s1);
    println!("in2 len: {}", in2.len());
    let (s2, in3) = util::forward(in2, &s2);
    println!("in3 len: {}", in3.len());

    // deaddrop
    let out3 = util::deaddrop(in3);
    println!("out3 len: {}", out3.len());

    // backward
    let out2 = util::backward(s2, out3);
    println!("out2 len: {}", out2.len());
    let out1 = util::backward(s1, out2);
    println!("out1 len: {}", out1.len());
    let out0 = util::backward(s0, out1);
    println!("out0 len: {}", out0.len());

    // unwrap and compare
    println!("try unwrap Alice...");
    let oa = client_util::unwrap(r, out0[0].clone(), &dka, server_dksa);
    println!("try unwrap Bob...");
    let ob = client_util::unwrap(r, out0[1].clone(), &dkb, server_dksb);
    println!("try unwrap Charlie...");
    let oc = client_util::unwrap(r, out0[2].clone(), &dkc, server_dksc);
    
    let ra = std::str::from_utf8(&oa).unwrap().trim_end_matches(0 as char);
    let rb = std::str::from_utf8(&ob).unwrap().trim_end_matches(0 as char);
    let rc = std::str::from_utf8(&oc).unwrap().trim_end_matches(0 as char);

    assert_eq!((ra, rb, rc), ("Hello, Alice!", "Hello, Bob!", "Hello, Charlie!"));
}
