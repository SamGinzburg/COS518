use crate::onion;

use std::{fs, io, path::PathBuf};

/// Read and write keys to hard-coded locations

pub enum PartyType {
    Client,
    Server,
}

pub struct Party {
    party_type: PartyType,
    id: usize,
}

impl PartyType {
    pub fn with_id(self, id: usize) -> Party {
        Party {
            party_type: self,
            id,
        }
    }
}

enum KeyType {
    Public,
    Private,
}

fn parent(t: &PartyType) -> PathBuf {
    let mut path = PathBuf::new();
    path.push("./keys");
    path.push(match t {
        PartyType::Client => "client",
        PartyType::Server => "server",
    });
    path
}

fn path(p: &Party, k: KeyType) -> PathBuf {
    let mut path = parent(&p.party_type);
    path.push(p.id.to_string());
    let e = match k {
        KeyType::Public => "pk",
        KeyType::Private => "sk",
    };
    path.set_extension(e);
    path
}

pub fn makedirs() -> io::Result<()> {
    fs::create_dir_all(parent(&PartyType::Client))?;
    fs::create_dir_all(parent(&PartyType::Server))?;
    Ok(())
}

pub fn put(s: Party, (sk, pk): onion::KeyPair) -> io::Result<()> {
    fs::write(path(&s, KeyType::Public), pk)?;
    fs::write(path(&s, KeyType::Private), sk)?;
    Ok(())
}

pub fn get(s: Party) -> io::Result<onion::PublicKey> {
    fs::read(path(&s, KeyType::Public))
}

pub fn get_keypair(s: Party) -> io::Result<onion::KeyPair> {
    let pk = fs::read(path(&s, KeyType::Public))?;
    let sk = fs::read(path(&s, KeyType::Private))?;
    Ok((sk, pk))
}
