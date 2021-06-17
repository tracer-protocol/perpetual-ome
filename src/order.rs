//! Contains logic and type definitions for orders
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

use chrono::{DateTime, NaiveDateTime, ParseError, Utc};
use derive_more::Display;
use ethabi::Token;
use hex::FromHexError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use web3::types::{Address, H256, U256};

pub type OrderId = H256;

/// Represents which side of the market an order is on
///
/// This type is simply an enum with two fields:
/// - `Bid` (buy-side)
/// - `Ask` (sell-side)
#[derive(
    Clone, Copy, PartialEq, Eq, Debug, Display, Serialize, Deserialize,
)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl FromStr for OrderSide {
    type Err = OrderParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Bid" | "bid" | "BID" => Ok(OrderSide::Bid),
            "Ask" | "ask" | "ASK" => Ok(OrderSide::Ask),
            _ => Err(OrderParseError::InvalidSide),
        }
    }
}

/// Represents an actual order in the market
///
/// Comprises a struct with all order fields needed for the Tracer market.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub trader: Address,
    pub market: Address,
    pub side: OrderSide,
    pub price: U256,
    pub quantity: U256,
    pub remaining: U256,
    pub expiration: DateTime<Utc>,
    pub created: DateTime<Utc>,
    pub signed_data: Vec<u8>,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "<ID:{} Market: {} Side: {} Price: {} Quantity: {} Remaining: {}>",
            self.id,
            self.market,
            self.side,
            self.price,
            self.quantity,
            self.remaining
        )
    }
}

/// Represents an error in interpreting a byte-level representation of an order
#[derive(Clone, Copy, Debug, Error, Serialize, Deserialize)]
pub enum OrderParseError {
    InvalidHexadecimal,
    InvalidSide,
    InvalidTimestamp,
    IntegerBounds,
    InvalidDecimal,
}

impl Display for OrderParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::InvalidHexadecimal => write!(f, "Invalid hexadecimal"),
            Self::InvalidSide => write!(f, "Invalid side"),
            _ => write!(f, "Unknown"),
        }
    }
}

impl From<FromHexError> for OrderParseError {
    fn from(_value: FromHexError) -> Self {
        OrderParseError::InvalidHexadecimal
    }
}

impl From<rustc_hex::FromHexError> for OrderParseError {
    fn from(_value: rustc_hex::FromHexError) -> Self {
        OrderParseError::InvalidHexadecimal
    }
}

impl From<ParseError> for OrderParseError {
    fn from(_value: ParseError) -> Self {
        OrderParseError::InvalidTimestamp
    }
}

impl From<ParseIntError> for OrderParseError {
    fn from(_value: ParseIntError) -> Self {
        OrderParseError::IntegerBounds
    }
}

pub fn order_id(
    user: Address,
    target_tracer: Address,
    side: OrderSide,
    price: U256,
    amount: U256,
    expiration: DateTime<Utc>,
    created: DateTime<Utc>,
) -> OrderId {
    /* handle indirect conversions */
    let side_num: U256 = U256::from(match side {
        OrderSide::Bid => 0u8,
        OrderSide::Ask => 1u8,
    });
    let expiration_timestamp: U256 = U256::from(expiration.timestamp());
    let created_timestamp: U256 = U256::from(created.timestamp());

    let components: Vec<Token> = vec![
        Token::Address(user),
        Token::Address(target_tracer),
        Token::Uint(price),
        Token::Uint(amount),
        Token::Uint(side_num),
        Token::Uint(expiration_timestamp),
        Token::Uint(created_timestamp),
    ];

    web3::signing::keccak256(&ethabi::encode(&components)).into()
}

impl Order {
    /// Constructor for the `Order` type
    ///
    /// Accepts all but the ID field of an order. Then calculates the order's ID
    /// and populates an `Order` struct.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        trader: Address,
        market: Address,
        side: OrderSide,
        price: U256,
        quantity: U256,
        expiration: DateTime<Utc>,
        created: DateTime<Utc>,
        signed_data: Vec<u8>,
    ) -> Self {
        let id: OrderId = order_id(
            trader, market, side, price, quantity, expiration, created,
        );

        Self {
            id,
            trader,
            market,
            side,
            price,
            quantity,
            remaining: quantity,
            expiration,
            created,
            signed_data,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct ExternalOrder {
    pub id: String,
    pub user: String,
    pub target_tracer: String,
    pub side: String,
    pub price: String,
    pub amount: String,
    pub amount_left: String,
    pub expiration: String,
    pub created: String,
    pub signed_data: String,
}

impl From<Order> for ExternalOrder {
    fn from(value: Order) -> Self {
        let id_bytes: Vec<u8> = value.id.as_ref().to_vec();
        let trader_bytes: Vec<u8> = value.trader.as_ref().to_vec();
        let market_bytes: Vec<u8> = value.market.as_ref().to_vec();
        Self {
            id: "0x".to_string() + &hex::encode(&id_bytes),
            user: "0x".to_string() + &hex::encode(&trader_bytes),
            target_tracer: "0x".to_string() + &hex::encode(&market_bytes),
            side: value.side.to_string(),
            price: value.price.to_string(),
            amount: value.quantity.to_string(),
            amount_left: value.remaining.to_string(),
            expiration: value.expiration.timestamp().to_string(),
            created: value.created.timestamp().to_string(),
            signed_data: "0x".to_string() + &hex::encode(value.signed_data),
        }
    }
}

impl TryFrom<ExternalOrder> for Order {
    type Error = OrderParseError;

    fn try_from(value: ExternalOrder) -> Result<Self, Self::Error> {
        let trader: Address = match Address::from_str(&value.user) {
            Ok(t) => t,
            Err(e) => return Err(e.into()),
        };

        let market: Address = match Address::from_str(&value.target_tracer) {
            Ok(t) => t,
            Err(e) => return Err(e.into()),
        };

        let side: OrderSide = match OrderSide::from_str(&value.side) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

        let price: U256 = match U256::from_dec_str(&value.price) {
            Ok(t) => t,
            Err(_e) => return Err(OrderParseError::InvalidDecimal),
        };

        let quantity: U256 = match U256::from_dec_str(&value.amount) {
            Ok(t) => t,
            Err(_e) => return Err(OrderParseError::InvalidDecimal),
        };

        let remaining: U256 = match U256::from_dec_str(&value.amount_left) {
            Ok(t) => t,
            Err(_e) => return Err(OrderParseError::InvalidDecimal),
        };

        let expiration: DateTime<Utc> = {
            let timestamp: i64 = match value.expiration.parse::<i64>() {
                Ok(t) => t,
                Err(_e) => return Err(OrderParseError::InvalidTimestamp),
            };

            DateTime::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc)
        };

        let created: DateTime<Utc> = {
            let timestamp: i64 = match value.created.parse::<i64>() {
                Ok(t) => t,
                Err(_e) => return Err(OrderParseError::InvalidTimestamp),
            };

            DateTime::from_utc(NaiveDateTime::from_timestamp(timestamp, 0), Utc)
        };

        let signed_data: Vec<u8> = match hex::decode(&value.signed_data) {
            Ok(t) => t,
            Err(e) => return Err(e.into()),
        };

        let id: OrderId = order_id(
            trader, market, side, price, quantity, expiration, created,
        );

        Ok(Self {
            id,
            trader,
            market,
            side,
            price,
            quantity,
            remaining,
            expiration,
            created,
            signed_data,
        })
    }
}
