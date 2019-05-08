#![feature(
    futures_api,
    arbitrary_self_types,
    await_macro,
    async_await,
    proc_macro_hygiene
)]

extern crate byteorder;
#[macro_use]
extern crate tarpc;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate ring;

pub mod client_util;
pub mod deaddrop_rpc;
pub mod head_rpc;
pub mod int_rpc;
pub mod keys;
pub mod laplace;
pub mod message;
pub mod onion;
pub mod permute;
pub mod util;

pub const NUM_CLIENTS: usize = 1000;
pub const NUM_SERVERS: usize = 3;
