#![feature(futures_api, arbitrary_self_types, await_macro, async_await, proc_macro_hygiene)]

#[macro_use] extern crate tarpc;
#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate ring;

pub mod onion;
pub mod message;
pub mod head_rpc;
pub mod int_rpc;
pub mod deaddrop_rpc;
pub mod keys;
pub mod permute;
pub mod laplace;
pub mod util;
pub mod client_util;

pub const NUM_CLIENTS : usize = 1000;
pub const NUM_SERVERS : usize = 3;
