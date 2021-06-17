use std::collections::HashMap;
use std::convert::{From, Infallible};
use std::sync::Arc;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use ethereum_types::{Address, U256};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use warp::http;
use warp::http::StatusCode;
use warp::reply::json;
use warp::{Rejection, Reply};

use crate::book::Book;
use crate::order::{Order, OrderId, OrderSide};
use crate::rpc;
use crate::state::OmeState;
use crate::util::{from_hex_de, from_hex_se};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OmeResponse {
    pub status: u16, /* this should be the StatusCode type, but serde */
    pub message: String,
}

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
    #[serde(with = "ts_seconds")]
    created: DateTime<Utc>, /* creation time of the order */
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
        let created: DateTime<Utc> = value.created;
        let signed_data: Vec<u8> = value.signed_data;

        /* construct order */
        Order::new(
            user,
            target_tracer,
            side,
            price,
            amount,
            expiration,
            created,
            signed_data,
        )
    }
}

pub type UpdateOrderRequest = CreateOrderRequest;

/// HEALTH POINT HANDLER
pub async fn health_check_handler() -> Result<impl Reply, Infallible> {
    let status: StatusCode = http::StatusCode::OK;
    let resp_body: OmeResponse = OmeResponse {
        status: status.as_u16(),
        message: "Healthy".to_string(),
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&resp_body),
        http::StatusCode::OK,
    ))
}

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
        let status: StatusCode = StatusCode::CONFLICT;
        let resp_body: OmeResponse = OmeResponse {
            status: status.as_u16(),
            message: "Market already exists".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&resp_body),
            status,
        ));
    }

    /* add the new book to the engine state */
    ome_state.add_book(new_book);

    info!("Created book {}", market);

    /* indicate success to the caller */
    let status: StatusCode = http::StatusCode::CREATED;
    let resp_body: OmeResponse = OmeResponse {
        status: status.as_u16(),
        message: "Market created".to_string(),
    };
    Ok(warp::reply::with_status(
        warp::reply::json(&resp_body),
        status,
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
        let status: StatusCode = http::StatusCode::BAD_REQUEST;
        let resp_body: OmeResponse = OmeResponse {
            status: status.as_u16(),
            message: "Integer out of bounds".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&resp_body),
            status,
        ));
    }

    let new_order: Order = Order::from(request);

    info!("Creating order {}...", new_order);

    let valid_order: bool = match rpc::check_order_validity(
        new_order.clone(),
        rpc_endpoint.clone(),
    )
    .await
    {
        Ok(t) => t,
        Err(_e) => false,
    };

    if !valid_order {
        let status: StatusCode = warp::http::StatusCode::BAD_REQUEST;
        let resp_body: OmeResponse = OmeResponse {
            status: status.as_u16(),
            message: "Invalid order".to_string(),
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&resp_body),
            status,
        ));
    }

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
            let status: StatusCode = warp::http::StatusCode::NOT_FOUND;
            let resp_body: OmeResponse = OmeResponse {
                status: status.as_u16(),
                message: "Market does not exist".to_string(),
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&resp_body),
                status,
            ));
        }
    };

    let tmp_order: Order = new_order.clone();

    /* submit order to the engine for matching */
    match book.submit(new_order, rpc_endpoint).await {
        Ok(order_status) => {
            info!("Created order {}", tmp_order);
            Ok(warp::reply::with_status(
                warp::reply::json(&order_status.to_string()),
                http::StatusCode::OK,
            ))
        }
        Err(e) => {
            warn!("Failed to create order {}! Engine said: {}", tmp_order, e);
            let status: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;
            let resp_body: OmeResponse = OmeResponse {
                status: status.as_u16(),
                message: "Matching error occurred".to_string(),
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&resp_body),
                status,
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
            let status: StatusCode = warp::http::StatusCode::NOT_FOUND;
            let resp_body: OmeResponse = OmeResponse {
                status: status.as_u16(),
                message: "Market does not exist".to_string(),
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&resp_body),
                status,
            ));
        }
    };

    /* retrieve order */
    let order: &Order = match book.order(id) {
        Some(o) => o,
        None => {
            let status: StatusCode = warp::http::StatusCode::NOT_FOUND;
            let resp_body: OmeResponse = OmeResponse {
                status: status.as_u16(),
                message: "Order does not exist in this market".to_string(),
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&resp_body),
                status,
            ));
        }
    };

    Ok(warp::reply::with_status(json(order), StatusCode::OK))
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
                warp::reply::json(&"Market does not exist".to_string()),
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
                warp::reply::json(
                    &"Order does not exist in market".to_string(),
                ),
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    let status: StatusCode = http::StatusCode::OK;
    let resp_body: OmeResponse = OmeResponse {
        status: status.as_u16(),
        message: "Order cancelled".to_string(),
    };
    Ok(
        warp::reply::with_status(warp::reply::json(&resp_body), status)
            .into_response(),
    )
}

#[allow(clippy::into_iter_on_ref)]
pub async fn market_user_orders_handler(
    market: Address,
    user: Address,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book */
    let book: &mut Book = match ome_state.book_mut(market) {
        Some(b) => b,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&"Market does not exist".to_string()),
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    let bids: Vec<Order> = book
        .bids
        .values()
        .into_iter()
        .flat_map(|levels| levels.into_iter().filter(|o| o.user == user))
        .cloned()
        .collect();

    let asks: Vec<Order> = book
        .asks
        .values()
        .into_iter()
        .flat_map(|levels| levels.into_iter().filter(|o| o.user == user))
        .cloned()
        .collect();

    let orders: Vec<Order> =
        bids.iter().cloned().chain(asks.iter().cloned()).collect();

    Ok(json(&orders).into_response())
}
