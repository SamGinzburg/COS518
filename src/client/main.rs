extern crate clap;

use clap::{App};

fn main() {
    App::new("Vuvuzela Client")
         .version("1.0")
         .about("Vuvuzela Client")
         .author("Sam Ginzburg")
         .get_matches();
}
