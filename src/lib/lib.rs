#![feature(futures_api, pin, arbitrary_self_types, await_macro, async_await, proc_macro_hygiene)]

#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate ring;
#[macro_use] extern crate tarpc;

pub mod onion;
pub mod message;
pub mod rpc;
pub mod keys;

pub const NUM_CLIENTS : usize = 1000;
pub const NUM_SERVERS : usize = 3;
