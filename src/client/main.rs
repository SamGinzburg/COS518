#![feature(futures_api, arbitrary_self_types, await_macro, async_await)]

extern crate clap;
extern crate sharedlib;
extern crate cursive;
extern crate tarpc_bincode_transport;

use clap::{App};
use cursive::Cursive;
use cursive::view::*;
use cursive::views::*;

mod send;

fn send_message(s: &mut Cursive, message: &str) {
    let mut text_area: ViewRef<TextView> = s.find_id("output").unwrap();
    let mut text_input: ViewRef<EditView> = s.find_id("input").unwrap();

    // clear the input
    text_input.set_content("");

    let mut input: String = "".to_string();
    input.push_str(&message.to_string());
    input.push_str("\n");
    text_area.append(input);
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

    let mut scrollbar = ScrollView::new(TextView::new("")
                                    .with_id("output"));

    scrollbar.set_scroll_strategy(ScrollStrategy::StickToBottom);

    cursive.add_layer(LinearLayout::vertical()
        .child(BoxView::new(SizeConstraint::Fixed(10),
                            SizeConstraint::Full,
                            Panel::new(scrollbar)))
        .child(text_box_view));

    // Starts the event loop.
    cursive.run();
}
