#![feature(map_first_last)]
#![feature(async_closure)]
#![feature(result_contains_err)]
#![feature(destructuring_assignment)]
use std::convert::{TryInto, TryFrom};
use std::sync::Arc;

use clap::{App, Arg};
use tokio::sync::Mutex;
use warp::Filter;

#[macro_use]
extern crate enum_display_derive;

#[macro_use]
extern crate log;
extern crate pretty_env_logger;

pub mod api;
pub mod args;
pub mod book;
pub mod handler;
pub mod order;
pub mod state;
pub mod tests;
pub mod util;
pub mod rpc;

#[cfg(test)]
pub mod book_tests;

use crate::args::Arguments;
use crate::order::{AddressWrapper, OrderId};
use crate::book::{Book};
use crate::state::OmeState;
use crate::rpc::{get_known_markets, get_external_book};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    /* define our command-line interface using Clap's builder syntax */
    let matches = App::new("Tracer OME")
        .version("0.1.0")
        .arg(
            Arg::with_name("address")
                .short("l")
                .long("listen")
                .value_name("address")
                .help("The address to listen on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .value_name("port")
                .help("The TCP port to listen on")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("certificate_path")
                .long("certificate_path")
                .value_name("certificate_path")
                .help("File path to the TLS certificate file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("private_key_path")
                .long("private_key_path")
                .value_name("private_key_path")
                .help("File path to the TLS private key file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("force-no-tls")
                .long("force-no-tls")
                .help("Flag to force TLS to be turned off"),
        )
        .arg(
            Arg::with_name("known_markets_url")
                .long("known_markets_url")
                .help("Endpoint to retrieve known market addresses from")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("external_book_url")
                .long("external_book_url")
                .help("Endpoint to retrieve external book by market id from")
                .takes_value(true),
        )
        .get_matches();

    let arguments: Arguments = match matches.try_into() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    let mut ome_state = OmeState::new();

    // restore market state
    // will panic and crash if this fails at all
    // fetch all markets known by the api
    let known_markets = get_known_markets(&arguments.known_markets_url).await.unwrap();

    // restore each of the known books
    for market_id in known_markets {
        let external_book = get_external_book(&arguments.external_book_url, market_id).await.unwrap();
        let book = Book::try_from(external_book);

        ome_state.add_book(book.unwrap());
    }

    /* initialise engine state */
    let state: Arc<Mutex<OmeState>> = Arc::new(Mutex::new(ome_state));

    /* Clone global engine state for each handler. This is only done because of
     * the nature of move semantics for Rust closures.
     *
     * This is quite hacky and likely quite expensive.
     */
    let index_book_state: Arc<Mutex<OmeState>> = state.clone();
    let create_book_state: Arc<Mutex<OmeState>> = state.clone();
    let read_book_state: Arc<Mutex<OmeState>> = state.clone();

    let create_order_state: Arc<Mutex<OmeState>> = state.clone();
    let read_order_state: Arc<Mutex<OmeState>> = state.clone();
    let destroy_order_state: Arc<Mutex<OmeState>> = state.clone();

    let market_user_orders_state: Arc<Mutex<OmeState>> = state.clone();

    /* define CRUD routes for order books */
    let book_prefix = warp::path!("book");
    let index_book_route = book_prefix
        .and(warp::get())
        .and(warp::any().map(move || index_book_state.clone()))
        .and_then(handler::index_book_handler);
    let create_book_route = book_prefix
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || create_book_state.clone()))
        .and_then(handler::create_book_handler);
    let read_book_route = warp::path!("book" / AddressWrapper)
        .and(warp::get())
        .and(warp::any().map(move || read_book_state.clone()))
        .and_then(handler::read_book_handler);

    /* define CRUD routes for orders */
    let create_order_route = warp::path!("book" / AddressWrapper / "order")
        .and(warp::post())
        .and(warp::body::json())
        .and(warp::any().map(move || create_order_state.clone()))
        .and_then(handler::create_order_handler);
    let read_order_route =
        warp::path!("book" / AddressWrapper / "order" / OrderId)
            .and(warp::get())
            .and(warp::any().map(move || read_order_state.clone()))
            .and_then(handler::read_order_handler);
    let destroy_order_route =
        warp::path!("book" / AddressWrapper / "order" / OrderId)
            .and(warp::delete())
            .and(warp::any().map(move || destroy_order_state.clone()))
            .and_then(handler::destroy_order_handler);

    let market_user_orders_route =
        warp::path!("book" / AddressWrapper / AddressWrapper)
            .and(warp::get())
            .and(warp::any().map(move || market_user_orders_state.clone()))
            .and_then(handler::market_user_orders_handler);

    // Healthcheck
    let health_route = warp::path::end()
        .and(warp::get())
        .and_then(handler::health_check_handler);

    /* aggregate all of our order book routes */
    let book_routes =
        index_book_route.or(create_book_route).or(read_book_route);

    /* aggregate all of our order routes */
    let order_routes = create_order_route
        .or(read_order_route)
        .or(destroy_order_route);

    let misc_routes = market_user_orders_route;

    let cors = warp::cors()
        .allow_any_origin()
        .allow_headers(vec![
            "Origin",
            "Access-Control-Request-Headers",
            "Content-Type",
            "Access-Control-Allow-Origin",
        ])
        .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

    /* aggregate all of our routes */
    let routes = health_route
        .or(book_routes)
        .or(order_routes)
        .or(misc_routes)
        .with(cors);

    /* start the web server */
    if arguments.force_no_tls {
        warp::serve(routes)
            .run((arguments.listen_address, arguments.listen_port))
            .await;
    } else {
        warp::serve(routes)
            .tls()
            .cert_path(arguments.certificate_path)
            .key_path(arguments.private_key_path)
            .run((arguments.listen_address, arguments.listen_port))
            .await;
    }
}
