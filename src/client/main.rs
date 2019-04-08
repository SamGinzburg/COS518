extern crate clap;
extern crate sharedlib;
extern crate cursive;

use clap::{App};
use cursive::Cursive;

fn main() {
    App::new("Vuvuzela Client")
         .version("1.0")
         .about("Vuvuzela Client")
         .author("Sam Ginzburg")
         .author("Benjamin Kuykendall")
         .get_matches();

    // set up main TUI context
    let mut siv = Cursive::default();


    siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
                         .title("Cursive")
                         .button("Quit", |s| s.quit()));

    // Starts the event loop.
    siv.run();
}
