extern crate clap;
extern crate sharedlib;

use clap::{App};
use sharedlib::example_fn;

fn main() {
    App::new("Vuvuzela Client")
         .version("1.0")
         .about("Vuvuzela Client")
         .author("Sam Ginzburg")
         .author("Benjamin Kuykendall")
         .get_matches();
    example_fn();
}
