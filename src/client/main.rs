extern crate clap;
extern crate sharedlib;
extern crate cursive;

use clap::{App};
use cursive::Cursive;
use cursive::view::*;
use cursive::views::*;
use cursive::event::*;

fn send_message(s: &mut Cursive, name: &str) {
    if name.is_empty() {
    } else {
    }
}

fn main() {
    App::new("Vuvuzela Client")
         .version("1.0")
         .about("Vuvuzela Client")
         .author("Sam Ginzburg")
         .author("Benjamin Kuykendall")
         .get_matches();

    // set up main TUI context
    let mut cursive = Cursive::default();

    // 
    // Create a view tree with a TextArea for input, and a
    // TextView for output.
    //

    let mut text_area = EditView::new()
                        .with_id("input");
    text_area.get_mut().set_on_submit(send_message);

    let text_box_view = BoxView::new(SizeConstraint::Full, SizeConstraint::Free,
                                     Panel::new(text_area));


    cursive.add_layer(LinearLayout::vertical()
        .child(BoxView::new(SizeConstraint::Fixed(10),
                            SizeConstraint::Full,
                            Panel::new(TextView::new("")
                                .with_id("output"))))
        .child(text_box_view));

    // Starts the event loop.
    cursive.run();
}
