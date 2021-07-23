#![allow(dead_code)]
use ethereum_types::Address;
use serde::{Deserialize, Serialize};

use crate::book::{Book, Fill, Fills, MatchResult, OrderStatus};
use crate::order::Order;

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
pub enum MessagePayload {
    Book(Book),
    Order(Order),
    String(String),
    Empty(()),
    Fills(Fills),
    Books(Vec<Address>),
    Orders(Vec<Order>),
}

#[derive(Clone, Debug, Serialize)]
pub struct Message {
    pub message: String,
    pub data: MessagePayload,
}

impl From<MatchResult> for outbound::Message {
    fn from(match_result: MatchResult) -> Self {
        match match_result.order_status {
            OrderStatus::Placed => outbound::Message::Placed,
            OrderStatus::PartialMatch => {
                outbound::Message::PartialMatch(match_result.fills)
            }
            OrderStatus::FullMatch => {
                outbound::Message::FullMatch(match_result.fills)
            }
        }
    }
}

impl From<outbound::Message> for Message {
    fn from(msg: outbound::Message) -> Self {
        match msg {
            outbound::Message::Placed => Self {
                message: "Placed".to_string(),
                data: MessagePayload::Empty(()),
            },
            outbound::Message::PartialMatch(fills) => Self {
                message: "Partially Matched".to_string(),
                data: MessagePayload::Fills(fills),
            },
            outbound::Message::FullMatch(fills) => Self {
                message: "Fully Matched".to_string(),
                data: MessagePayload::Fills(fills),
            },
            outbound::Message::Cancelled => Self {
                message: "Cancelled".to_string(),
                data: MessagePayload::Empty(()),
            },
            outbound::Message::ReadBook(book) => Self {
                message: "Book".to_string(),
                data: MessagePayload::Book(book),
            },
            outbound::Message::ReadOrder(order) => Self {
                message: "Order".to_string(),
                data: MessagePayload::Order(order),
            },
            outbound::Message::Error(e) => Self {
                message: "Error".to_string(),
                data: MessagePayload::String(e.to_string()),
            },
            outbound::Message::BookCreated => Self {
                message: "Book Created".to_string(),
                data: MessagePayload::Empty(()),
            },
            outbound::Message::OrderCreated => Self {
                message: "Order Created".to_string(),
                data: MessagePayload::Empty(()),
            },
            outbound::Message::ListBooks(books) => Self {
                message: "Books".to_string(),
                data: MessagePayload::Books(books),
            },
            outbound::Message::ListOrders(orders) => Self {
                message: "Orders".to_string(),
                data: MessagePayload::Orders(orders),
            },
            outbound::Message::OrderDestroyed => Self {
                message: "Order Destroyed".to_string(),
                data: MessagePayload::Empty(()),
            },
            outbound::Message::BookDestroyed => Self {
                message: "Book Destroyed".to_string(),
                data: MessagePayload::Empty(()),
            },
        }
    }
}

pub mod inbound {
    use super::*;
    use chrono::serde::ts_seconds;
    use chrono::{DateTime, Utc};
    use ethereum_types::{Address, U256};

    use crate::order::{OrderId, OrderSide};

    /// Represents an API request to create a new order
    #[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct CreateOrderRequest {
        user: Address,          /* Ethereum address of trader */
        target_tracer: Address, /* Ethereum address of the Tracer smart contract */
        side: OrderSide,        /* side of the market of the order */
        price: U256,            /* price */
        amount: U256,           /* quantity */
        #[serde(with = "ts_seconds")]
        expiration: DateTime<Utc>, /* expiration of the order */
        #[serde(with = "ts_seconds")]
        created: DateTime<Utc>, /* creation time of the order */
        signed_data: String,    /* digital signature of the order */
    }

    pub type UpdateOrderRequest = CreateOrderRequest;

    #[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct CreateBookRequest {
        address: Address,
    }

    #[derive(Clone, Debug)]
    pub enum Message {
        CreateOrder(CreateOrderRequest),
        ReadOrder(OrderId),
        DeleteOrder(OrderId),
        CreateBook(CreateBookRequest),
        ReadBook(Address),
        DeleteBook(Address),
    }
}

pub mod outbound {
    use super::*;

    use std::fmt;
    use std::fmt::{Display, Formatter};

    use crate::book::Book;

    pub type Fills = Vec<Fill>;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub enum Error {
        NoSuchBook,
        NoSuchOrder,
        InvalidOrder,
        BookExists,
    }

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter) -> fmt::Result {
            match self {
                Self::NoSuchBook => write!(f, "No such book"),
                Self::NoSuchOrder => write!(f, "No such order"),
                Self::InvalidOrder => write!(f, "Invalid order"),
                Self::BookExists => write!(f, "Book already exists"),
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum Message {
        Placed,
        PartialMatch(Fills),
        FullMatch(Fills),
        Cancelled,
        Error(Error),
        ReadBook(Book),
        ReadOrder(Order),
        BookCreated,
        OrderCreated,
        ListBooks(Vec<Address>),
        ListOrders(Vec<Order>),
        BookDestroyed,
        OrderDestroyed,
    }
}
