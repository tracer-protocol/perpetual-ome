use std::convert::{From, Infallible, TryFrom};
use std::sync::Arc;

use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use ethereum_types::{Address, H256, U256};
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};
use warp::http;
use warp::http::StatusCode;
use warp::reply::json;
use warp::{Rejection, Reply};

use crate::api;
use crate::book::Book;
use crate::order::{AddressWrapper, ExternalOrder, Order, OrderId, OrderSide};
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
    signed_data: String,    /* digital signature of the order */
}

impl From<CreateOrderRequest> for ExternalOrder {
    fn from(value: CreateOrderRequest) -> Self {
        /* extract request fields */
        let user: Address = value.user;
        let target_tracer: Address = value.target_tracer;
        let side: OrderSide = value.side;
        let price: U256 = value.price;
        let amount: U256 = value.amount;
        let expiration: DateTime<Utc> = value.expiration;
        let created: DateTime<Utc> = value.created;
        let signed_data: String = value.signed_data;

        let user_bytes: Vec<u8> = user.as_ref().to_vec();
        let target_tracer_bytes: Vec<u8> = target_tracer.as_ref().to_vec();

        let order: ExternalOrder = Self {
            id: hex::encode(H256::zero().as_ref().to_vec()),
            user: hex::encode(&user_bytes),
            target_tracer: hex::encode(&target_tracer_bytes),
            side: side.to_string(),
            price: price.to_string(),
            amount: amount.to_string(),
            amount_left: amount.to_string(),
            expiration: expiration.timestamp().to_string(),
            created: created.timestamp().to_string(),
            signed_data: {
                let mut chr = signed_data.chars();
                chr.next();
                chr.next();
                chr.as_str().to_string()
            },
        };

        order
    }
}

pub type UpdateOrderRequest = CreateOrderRequest;

/// HEALTH POINT HANDLER
pub async fn health_check_handler() -> Result<impl Reply, Infallible> {
    let msg: api::Message = api::Message {
        message: "Healthy".to_string(),
        data: api::MessagePayload::Empty(()),
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&msg),
        http::StatusCode::OK,
    ))
}

/// REST API route handler for listing all order books
pub async fn index_book_handler(
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Infallible> {
    let ome_state: MutexGuard<OmeState> = state.lock().await;

    let msg: api::Message =
        api::Message::from(api::outbound::Message::ListBooks(
            ome_state.books().keys().cloned().collect(),
        ));

    Ok(json(&msg))
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
        let msg: api::Message = api::Message::from(
            api::outbound::Message::Error(api::outbound::Error::BookExists),
        );

        return Ok(warp::reply::with_status(warp::reply::json(&msg), status));
    }

    /* add the new book to the engine state */
    ome_state.add_book(new_book);

    info!("Created book {}", market);

    /* indicate success to the caller */
    let status: StatusCode = http::StatusCode::CREATED;
    let msg: api::Message =
        api::Message::from(api::outbound::Message::BookCreated);

    Ok(warp::reply::with_status(warp::reply::json(&msg), status))
}

/// REST API route handler for retrieving a single order book
pub async fn read_book_handler(
    market: AddressWrapper,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let msg: api::Message = api::Message::from(
        match state.lock().await.book(Address::from(market)) {
            Some(t) => api::outbound::Message::ReadBook(t.clone()),
            None => {
                api::outbound::Message::Error(api::outbound::Error::NoSuchBook)
            }
        },
    );

    Ok(json(&msg).into_response())
}

/// REST API route handler for creating a single order
pub async fn create_order_handler(
    market: AddressWrapper,
    request: CreateOrderRequest,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let new_order: ExternalOrder = ExternalOrder::from(request);

    let internal_order: Order = match Order::try_from(new_order.clone()) {
        Ok(t) => t,
        Err(_e) => {
            let status: StatusCode = StatusCode::BAD_REQUEST;
            let msg: api::Message =
                api::Message::from(api::outbound::Message::Error(
                    api::outbound::Error::InvalidOrder,
                ));
            return Ok(warp::reply::with_status(
                warp::reply::json(&msg),
                status,
            ));
        }
    };

    info!("Creating order {}...", internal_order.clone());

    /* acquire lock on global state */
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book from global state */
    let book: &mut Book = match ome_state.book_mut(Address::from(market)) {
        Some(b) => b,
        None => {
            warn!(
                "Failed to create order {:?} as market does not exist!",
                new_order
            );
            let status: StatusCode = warp::http::StatusCode::NOT_FOUND;
            let msg: api::Message =
                api::Message::from(api::outbound::Message::BookNotFound);
            return Ok(warp::reply::with_status(
                warp::reply::json(&msg),
                status,
            ));
        }
    };

    /* submit order to the engine for matching */
    match book
        .submit(Order::try_from(new_order.clone()).unwrap())
        .await
    {
        Ok(match_result) => {
            info!("Created order {}", internal_order.clone());
            let status: StatusCode = StatusCode::OK;
            let msg: api::Message =
                api::Message::from(api::outbound::Message::from(match_result));
            Ok(warp::reply::with_status(warp::reply::json(&msg), status))
        }
        Err(e) => {
            warn!("Failed to create order {:?}! Engine said: {}", new_order, e);
            let status: StatusCode = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(warp::reply::with_status(warp::reply::json(&()), status))
        }
    }
}

/// REST API route handler for retrieving a single order
pub async fn read_order_handler(
    market: AddressWrapper,
    id: OrderId,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let ome_state: MutexGuard<OmeState> = state.lock().await;

    let msg: api::Message =
        api::Message::from(match ome_state.book(Address::from(market)) {
            Some(book) => match book.order(id) {
                Some(order) => api::outbound::Message::ReadOrder(order.clone()),
                None => api::outbound::Message::Error(
                    api::outbound::Error::NoSuchOrder,
                ),
            },
            None => {
                api::outbound::Message::Error(api::outbound::Error::NoSuchBook)
            }
        });

    Ok(warp::reply::with_status(json(&msg), StatusCode::OK))
}

/// REST API route handler for deleting a single order
///
/// Note that this is equivalent to order cancellation
pub async fn destroy_order_handler(
    market: AddressWrapper,
    id: OrderId,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book */
    let book: &mut Book = match ome_state.book_mut(Address::from(market)) {
        Some(b) => b,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&api::Message::from(
                    api::outbound::Message::Error(
                        api::outbound::Error::NoSuchBook,
                    ),
                )),
                http::StatusCode::OK,
            )
            .into_response());
        }
    };

    /* cancel order */
    match book.cancel(id) {
        Ok(_t) => {}
        Err(_e) => {
            let msg: api::Message =
                api::Message::from(api::outbound::Message::Error(
                    api::outbound::Error::NoSuchOrder,
                ));
            return Ok(warp::reply::with_status(
                warp::reply::json(&msg),
                http::StatusCode::NOT_FOUND,
            )
            .into_response());
        }
    };

    let status: StatusCode = http::StatusCode::OK;
    let msg: api::Message =
        api::Message::from(api::outbound::Message::OrderDestroyed);
    Ok(warp::reply::with_status(warp::reply::json(&msg), status)
        .into_response())
}

#[allow(clippy::into_iter_on_ref)]
pub async fn market_user_orders_handler(
    market: AddressWrapper,
    user: AddressWrapper,
    state: Arc<Mutex<OmeState>>,
) -> Result<impl Reply, Rejection> {
    let mut ome_state: MutexGuard<OmeState> = state.lock().await;

    /* retrieve order book */
    let book: &mut Book = match ome_state.book_mut(Address::from(market)) {
        Some(b) => b,
        None => {
            let status: StatusCode = StatusCode::NOT_FOUND;
            let msg: api::Message = api::Message::from(
                api::outbound::Message::Error(api::outbound::Error::NoSuchBook),
            );
            return Ok(warp::reply::with_status(
                warp::reply::json(&msg),
                status,
            )
            .into_response());
        }
    };

    let bids: Vec<Order> = book
        .bids
        .values()
        .into_iter()
        .flat_map(|levels| {
            levels
                .into_iter()
                .filter(|o| o.trader == Address::from(user))
        })
        .cloned()
        .collect();

    let asks: Vec<Order> = book
        .asks
        .values()
        .into_iter()
        .flat_map(|levels| {
            levels
                .into_iter()
                .filter(|o| o.trader == Address::from(user))
        })
        .cloned()
        .collect();

    let orders: Vec<ExternalOrder> = bids
        .iter()
        .cloned()
        .chain(asks.iter().cloned())
        .map(ExternalOrder::from)
        .collect();

    let status: StatusCode = http::StatusCode::OK;
    let msg: api::Message =
        api::Message::from(api::outbound::Message::ListOrders(orders));
    Ok(warp::reply::with_status(warp::reply::json(&msg), status)
        .into_response())
}
