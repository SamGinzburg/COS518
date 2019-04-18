#![feature(futures_api, pin, arbitrary_self_types, await_macro, async_await, proc_macro_hygiene)]

#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate ring;
#[macro_use] extern crate tarpc;

pub mod onion;
pub mod rpc;

