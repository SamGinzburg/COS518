#![feature(futures_api, pin, arbitrary_self_types, await_macro, async_await, proc_macro_hygiene)]

#[macro_use] extern crate tarpc;
extern crate openssl;
extern crate rand;

pub mod onion;
pub mod rpc;

pub fn example_fn() {
    println!("testing123");
}
