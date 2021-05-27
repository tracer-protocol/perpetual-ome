use std::collections::HashMap;
use std::convert::{From, Infallible};
use std::sync::Arc;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use ethereum_types::{Address, U256};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use warp::http;
use warp::reply::json;
use warp::{Rejection, Reply};

use crate::book::Book;
use crate::order::{Order, OrderId, OrderSide};
use crate::state::OmeState;
use crate::util::{from_hex_de, from_hex_se};

/// Represents an API request to create a new order book
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBookRequest {
    market: Address,
}

/// Represents an API request to create a new order
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct CreateOrderRequest {
    user: Address,          /* Ethereum address of trader */
    target_tracer: Address, /* Ethereum address of the Tracer smart contract */
    side: OrderSide,        /* side of the market of the order */
    #[serde(serialize_with = "from_hex_se", deserialize_with = "from_hex_de")]
    price: U256, /* price */
    #[serde(serialize_with = "from_hex_se", deserialize_with = "from_hex_de")]
    amount: U256, /* quantity */
    #[serde(with = "ts_seconds")]
    expiration: DateTime<Utc>, /* expiration of the order */
    signed_data: Vec<u8>,   /* digital signature of the order */
}

impl From<CreateOrderRequest> for Order {
    fn from(value: CreateOrderRequest) -> Self {
        /* extract request fields */
        let user: Address = value.user;
        let target_tracer: Address = value.target_tracer;
        let side: OrderSide = value.side;
        let price: U256 = value.price;
        let amount: U256 = value.amount;
        let expiration: DateTime<Utc> = value.expiration;
        let signed_data: Vec<u8> = value.signed_data;

        /* construct order */
        Order::new(
            user,
            target_tracer,
            side,
            price,
            amount,
            expiration,
            signed_data,
        )
    }
}

pub type UpdateOrderRequest = CreateOrderRequest;

/// REST API route handler for listing all order books
pub async fn index_book_handler(
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Infallible> {
    let ome_state: MutexGuard<OmeState> = state.lock().await;

    let mut result: HashMap<String, Vec<Address>> = HashMap::new();
    result.insert(
        "markets".to_string(),
        ome_state.books().keys().cloned().collect(),
    );

    Ok(json(&result))
}

/// REST API route handler for creating new order books
pub async fn create_book_handler(
    request: CreateBookRequest,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    /* build our new order book */
    let market: Address = request.market;
    let new_book: Book = Book::new(market);

    info!("Creating book {}...", market);

    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* check if the market already exists and, if so, return HTTP 409 */
    if ome_state.book(market).is_some() {
        return Ok(warp::reply::with_status(
            "Market already exists",
            http::StatusCode::CONFLICT,
        ));
    }

    /* add the new book to the engine state */
    ome_state.add_book(new_book);

    info!("Created book {}", market);

    /* indicate success to the caller */
    Ok(warp::reply::with_status(
        "Created new market",
        http::StatusCode::CREATED,
    ))
}

/// REST API route handler for retrieving a single order book
pub async fn read_book_handler(
    market: Address,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let ome_state: MutexGuard<OmeState> = state.lock().await;
    Ok(json(&ome_state.book(market)))
}

/// REST API route handler for creating a single order
pub async fn create_order_handler(
    market: Address,
    request: CreateOrderRequest,
    state: Arc<Mutex<OmeState>>,
    rpc_endpoint: String,
) -> Result<impl Reply, Rejection> {
    /* bounds check price and amount */
    if request.price > U256::from(u128::MAX)
        || request.amount > U256::from(u128::MAX)
    {
        return Ok(warp::reply::with_status(
            "Integers out of bounds",
            http::StatusCode::BAD_REQUEST,
        ));
    }

    let new_order: Order = Order::from(request);

    info!("Creating order {}...", new_order);

    /* acquire lock on global state */
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book from global state */
    let book: &mut Book = match ome_state.book_mut(market) {
        Some(b) => b,
        None => {
            warn!(
                "Failed to create order {} as market does not exist!",
                new_order
            );
            return Ok(warp::reply::with_status(
                "Market does not exist",
                http::StatusCode::NOT_FOUND,
            ));
        }
    };

    let tmp_order: Order = new_order.clone();

    /* submit order to the engine for matching */
    match book.submit(new_order, rpc_endpoint).await {
        Ok(_) => {
            info!("Created order {}", tmp_order);
            Ok(warp::reply::with_status("", http::StatusCode::OK))
        }
        Err(e) => {
            warn!("Failed to create order {}! Engine said: {}", tmp_order, e);
            Ok(warp::reply::with_status(
                "",
                http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

/// REST API route handler for retrieving a single order
pub async fn read_order_handler(
    market: Address,
    id: OrderId,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book */
    let book: &Book = match ome_state.book(market) {
        Some(b) => b,
        None => {
            return Ok(warp::reply::with_status(
                "Market does not exist",
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    /* retrieve order */
    let order: &Order = match book.order(id) {
        Some(o) => o,
        None => {
            return Ok(warp::reply::with_status(
                "Order does not exist in market",
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    Ok(json(order).into_response())
}

/// REST API route handler for updating a single order
///
/// Note that this is actually just a convenience method for usability's sake -
/// there's no such thing as actually mutating an order in-place in the Tracer
/// protocol (think about how digital signatures work)
pub async fn update_order_handler(
    market: Address,
    id: OrderId,
    request: UpdateOrderRequest,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book */
    let book: &mut Book = match ome_state.book_mut(market) {
        Some(b) => b,
        None => {
            return Ok(warp::reply::with_status(
                "Market does not exist",
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    /* retrieve order */
    let order: &mut Order = match book.order_mut(id) {
        Some(o) => o,
        None => {
            return Ok(warp::reply::with_status(
                "Order does not exist in market",
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    *order.price_mut() = request.price;
    *order.amount_mut() = request.amount;
    *order.expiration_mut() = request.expiration;

    Ok(warp::reply::with_status("", http::StatusCode::OK).into_response())
}

/// REST API route handler for deleting a single order
///
/// Note that this is equivalent to order cancellation
pub async fn destroy_order_handler(
    market: Address,
    id: OrderId,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book */
    let book: &mut Book = match ome_state.book_mut(market) {
        Some(b) => b,
        None => {
            return Ok(warp::reply::with_status(
                "Market does not exist",
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    /* cancel order */
    match book.cancel(id) {
        Ok(_t) => {}
        Err(_e) => {
            return Ok(warp::reply::with_status(
                "Order does not exist in market",
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    Ok(warp::reply::with_status("", http::StatusCode::OK).into_response())
}
