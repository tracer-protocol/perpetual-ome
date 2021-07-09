#![allow(dead_code)]
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    message: String,
    data: Option<String>,
}

impl From<outbound::Message> for Message {
    fn from(msg: outbound::Message) -> Self {
        match msg {
            outbound::Message::Placed => Self { message: "Placed".to_string(), data: None },
            outbound::Message::PartialMatch(fills) => Self { message: "Partially Matched".to_string(), data: fills.to_string() },
            outbound::Message::FullMatch(fills) => Self { message: "Fully Matched".to_string(), data: Somefills.to_string()) },
            outbound::Message::Cancelled(id) => Self { message: "Cancelled".to_string(), data: Some() },
        }
    }
}

pub mod inbound {
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

    pub type UpdateOrderRequest = CreateOrderRequest;

    #[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
    pub struct CreateBookRequest {
        address: Address,
    }

    pub type DeleteBookRequest = CreateBookRequest;

    #[derive(Clone, Debug)]
    pub enum Message {
        CreateOrder(CreateOrderRequest),
        ReadOrder(ReadOrderRequest),
        DeleteOrder(DeleteOrderRequest),
        CreateBook(CreateBookRequest),
        ReadBook,
        DeleteBook(DeleteBookRequest),
    }
}

pub mod outbound {
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum Error {
        NoSuchBook,
        NoSuchOrder,
        InvalidOrder,
    }

    #[derive(Clone, Debug)]
    pub enum Message {
        Placed,
        PartialMatch(Fills),
        FullMatch(Fills),
        Cancelled,
        Error(Error),
    }
}

