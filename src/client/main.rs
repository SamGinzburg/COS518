#![feature(type_ascription, generators, proc_macro_hygiene, futures_api, arbitrary_self_types, await_macro, async_await)]
#[macro_use] extern crate tarpc;
#[macro_use] extern crate lazy_static;

extern crate clap;
extern crate sharedlib;
extern crate cursive;
extern crate tarpc_bincode_transport;
extern crate futures;
extern crate serde;
extern crate futures_await_async_macro;
extern crate tokio;

use std::collections::HashMap;
use clap::{App, Arg};
use cursive::Cursive;
use cursive::view::*;
use cursive::views::*;

use crate::tarpc::futures::TryFutureExt;
use crate::tarpc::futures::FutureExt;
use crate::tarpc::futures::compat::Executor01CompatExt;
use crate::send::rpc_put;
use crate::fetch::rpc_get;
use std::{thread, time};

pub mod send;
pub mod fetch;
pub mod util;

lazy_static! {
    // quick hack to get args into callback function without modifying the 
    // cursive lib / making a custom UI object
    static ref HASHMAP: HashMap<String, String> = {
        let mut m = HashMap::new();
        let matches = App::new("Vuvuzela Client")
                        .version("1.0")
                        .about("Vuvuzela Client")
                        .author("Sam Ginzburg")
                        .author("Benjamin Kuykendall")
                        // this effectively corresponds to the id # for key lookup
                        .arg(Arg::with_name("name")
                            .short("n")
                            .long("name")
                            .help("Specifies the name of the user (used to lookup public/private key pair)")
                            .takes_value(true))
                        .arg(Arg::with_name("dial")
                            .short("d")
                            .long("dial")
                            .help("Specifies the name of the person you are dialing")
                            .takes_value(true))
                        .get_matches();

        // if these unwraps fail, we must panic!
        let uid = match matches.value_of("name") {
            Some(u) => String::from(u).clone(),
            None    => panic!("name not specified in CLI arguments"),
        };


        let remote_uid = match matches.value_of("dial") {
            Some(u) => String::from(u).clone(),
            None    => panic!("name not specified in CLI arguments"),
        };

        m.insert(String::from("uid"), uid);
        m.insert(String::from("remote_uid"), remote_uid);
        m.clone()
    };
}

fn send_message(s: &mut Cursive, message: &str) {
    let mut text_area: ViewRef<TextView> = s.find_id("output").unwrap();
    let mut text_input: ViewRef<EditView> = s.find_id("input").unwrap();

    // clear the input
    text_input.set_content("");

    let uid = HASHMAP.get(&String::from("uid")).unwrap().parse::<usize>().unwrap();
    let remote_uid = HASHMAP.get(&String::from("remote_uid")).unwrap().parse::<usize>().unwrap();

    let mut input: String = "".to_string();

    input.push_str(&message.to_string());
    input.push_str("\n");
    text_area.append(input.clone());

    // TODO: set ip/port combo via cli flags
    tokio::run(rpc_put("127.0.0.1".to_string(), 8080, input.clone(), uid, remote_uid)
               .map_err(|e| eprintln!("RPC Error: {}", e))
               .boxed()
               .compat(),
    );
}

fn main() {


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
                    .map_err(|e| {
                        eprintln!("Fetch Error: {}", e) })
                    .boxed()
                    .compat(),);
            thread::sleep(time::Duration::from_millis(1000));
        }
    });

    // Starts the event loop.
    cursive.run();
    handler.join().unwrap();
}
