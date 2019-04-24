#![feature(type_ascription, generators, proc_macro_hygiene, futures_api, arbitrary_self_types, await_macro, async_await)]
#[macro_use] extern crate tarpc;

extern crate clap;
extern crate sharedlib;
extern crate cursive;
extern crate tarpc_bincode_transport;
extern crate futures;
extern crate serde;
extern crate futures_await_async_macro;
extern crate tokio;

use clap::{App};
use cursive::Cursive;
use cursive::view::*;
use cursive::views::*;

use std::process::exit;

use crate::tarpc::futures::StreamExt;
use crate::tarpc::futures::TryFutureExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::compat::Executor01CompatExt;
use crate::send::rpc_put;
use crate::fetch::rpc_get;
use std::thread;

use tokio::prelude::future::{ok, loop_fn, Future, FutureResult, Loop};


pub mod send;
pub mod fetch;
mod lib;

fn send_message(s: &mut Cursive, message: &str) {
    let mut text_area: ViewRef<TextView> = s.find_id("output").unwrap();
    let mut text_input: ViewRef<EditView> = s.find_id("input").unwrap();

    // clear the input
    text_input.set_content("");

    let mut input: String = "".to_string();
    input.push_str(&message.to_string());
    input.push_str("\n");
    text_area.append(input.clone());

    // TODO: set ip/port combo via cli flags
    tokio::run(rpc_put("127.0.0.1".to_string(), 8080, input.clone())
               .map_err(|e| eprintln!("RPC Error: {}", e))
               .boxed()
               .compat(),
    );
}

fn main() {
    App::new("Vuvuzela Client")
         .version("1.0")
         .about("Vuvuzela Client")
         .author("Sam Ginzburg")
         .author("Benjamin Kuykendall")
         .get_matches();

    tarpc::init(tokio::executor::DefaultExecutor::current().compat());

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
 
    // start fetching data from server once GUI is initialized
    let handler = thread::spawn(|| {
        loop {
            tokio::run((rpc_get("127.0.0.1".to_string(), 8080))
                    .map_err(|e| eprintln!("Fetch Error: {}", e))
                    .boxed()
                    .compat(),);
        }
    });

    // Starts the event loop.
    cursive.run();
    handler.join().unwrap();
}
