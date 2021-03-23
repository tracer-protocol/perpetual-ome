//! Contains logic and type definitions for orders
use std::fmt;
use std::mem;

use byte_slice_cast::AsByteSlice;
use byteorder::{LittleEndian, WriteBytesExt};
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use thiserror::Error;
use web3::signing::recover;
use web3::types::{Address, Recovery, U256};

use crate::util::{from_hex_de, from_hex_se};

/// Magic string representing the function signature
pub const FUNCTION_SIGNATURE: &str = "LimitOrder(uint256 amount,uint256 price,bool side,address user,uint256 expiration,address targetTracer,uint256 nonce)";

/// Magic pre-computed hash of the EIP712 domain prefix
pub const DOMAIN_HASH: &str =
    "49854490ba36fba358fe1019f097d8b566d011cfb3fd67c6fce6a40624150034";

/// Magic number prefix for EIP712
pub const EIP712_MAGIC_PREFIX: &str = "1901";

pub type OrderId = u64;

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

impl OrderSide {
    /// Returns a byte slice of the market side
    ///
    /// This is simply one byte long as there will only ever be two market sides
    /// (realistically)
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            OrderSide::Bid => &[0x00],
            OrderSide::Ask => &[0x01],
        }
    }
}

/// Represents an actual order in the market
///
/// Comprises a struct with all order fields needed for the Tracer market.
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Order {
    id: u64,          /* SHA3-256 hash of other fields */
    address: Address, /* Ethereum address of trader */
    market: Address,  /* Ethereum address of the Tracer smart contract */
    side: OrderSide,  /* side of the market of the order */
    #[serde(serialize_with = "from_hex_se", deserialize_with = "from_hex_de")]
    price: U256, /* price */
    #[serde(serialize_with = "from_hex_se", deserialize_with = "from_hex_de")]
    amount: U256, /* quantity */
    #[serde(with = "ts_seconds")]
    expiration: DateTime<Utc>, /* expiration of the order */
    signed_data: Vec<u8>, /* digital signature of the order */
    nonce: U256,      /* order nonce */
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "#{} [{}] {} {} @ {}",
            self.id, self.market, self.side, self.amount, self.price
        )
    }
}

/// Represents an error in interpreting a byte-level representation of an order
#[derive(Clone, Copy, Debug, Error, Serialize, Deserialize)]
pub enum OrderParseError {/* TODO: add specific errors here */}

impl Order {
    /// Constructor for the `Order` type
    ///
    /// Accepts all but the ID field of an order. Then calculates the order's ID
    /// and populates an `Order` struct.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        address: Address,
        market: Address,
        side: OrderSide,
        price: U256,
        amount: U256,
        expiration: DateTime<Utc>,
        signed_data: Vec<u8>,
        nonce: U256,
    ) -> Self {
        let mut order_bytes: Vec<u8> = vec![]; /* this stores our hash input */

        /* address as bytes */
        order_bytes.extend_from_slice(address.as_byte_slice());

        /* market address as bytes */
        order_bytes.extend_from_slice(market.as_byte_slice());

        /* side as bytes */
        order_bytes.extend_from_slice(side.as_bytes());

        /* marshal expiration datetime into bytes by using Unix timestamp */
        let expiration_timestamp: i64 = expiration.timestamp();
        let mut expiration_bytes = [0u8; mem::size_of::<i64>()];
        expiration_bytes
            .as_mut()
            .write_i64::<LittleEndian>(expiration_timestamp)
            .unwrap();
        order_bytes.extend_from_slice(&expiration_bytes);

        /* price as bytes */
        order_bytes.extend_from_slice(price.as_byte_slice());

        /* amount as bytes */
        order_bytes.extend_from_slice(amount.as_byte_slice());

        let id: OrderId = 0; /* TODO: determine how IDs are to be generated */

        Self {
            id,
            address,
            market,
            side,
            price,
            amount,
            expiration,
            signed_data,
            nonce,
        }
    }

    /// Returns the unique identifier of this order
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Returns a mutable reference to the unique identifier of this order
    pub fn id_mut(&mut self) -> &mut u64 {
        &mut self.id
    }

    /// Returns the address of the owner of this order
    pub fn address(&self) -> Address {
        self.address
    }

    /// Returns a mutable reference to the address of the owner of this order
    pub fn address_mut(&mut self) -> &mut Address {
        &mut self.address
    }

    /// Returns the market of the market of this order
    pub fn market(&self) -> Address {
        self.market
    }

    /// Returns a mutable reference to the address of the market of this order
    pub fn market_mut(&mut self) -> &mut Address {
        &mut self.market
    }

    /// Returns the market side of this order
    pub fn side(&self) -> OrderSide {
        self.side
    }

    /// Returns a mutable reference to the market side of this order
    pub fn side_mut(&mut self) -> &mut OrderSide {
        &mut self.side
    }

    /// Returns the price of this order
    pub fn price(&self) -> U256 {
        self.price
    }

    /// Returns a mutable reference to the price of this order
    pub fn price_mut(&mut self) -> &mut U256 {
        &mut self.price
    }

    /// Returns the quantity of this order
    pub fn amount(&self) -> U256 {
        self.amount
    }

    /// Returns a mutable reference to the quantity of this order
    pub fn amount_mut(&mut self) -> &mut U256 {
        &mut self.amount
    }

    /// Returns the expiration of this order
    pub fn expiration(&self) -> DateTime<Utc> {
        self.expiration
    }

    /// Returns a mutable reference to the expiration of this order
    pub fn expiration_mut(&mut self) -> &mut DateTime<Utc> {
        &mut self.expiration
    }

    /// Returns the digital signature associated with this order
    pub fn signed_data(&self) -> &[u8] {
        self.signed_data.as_ref()
    }

    /// Returns a reference to the order's nonce
    pub fn nonce(&self) -> U256 {
        self.nonce
    }

    /// Returns a mutable reference to the order's nonce
    pub fn nonce_mut(&mut self) -> &mut U256 {
        &mut self.nonce
    }

    /// Serialises the order to its byte-level representation
    pub fn to_bytes(&self) -> Vec<u8> {
        /* Tracer contract expects boolean for order side */
        let side_flag: bool = match self.side() {
            OrderSide::Bid => true,
            OrderSide::Ask => false,
        };

        /* hash the magic function signature string */
        let function_signature_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(FUNCTION_SIGNATURE);
            hasher.finalize()
        };

        /* encode order into tokens (NOTE: order matters!) */
        let abi_tokens: Vec<ethabi::Token> = vec![
            ethabi::Token::FixedBytes(function_signature_hash.to_vec()),
            ethabi::Token::Uint(self.amount()),
            ethabi::Token::Uint(self.price()),
            ethabi::Token::Bool(side_flag),
            ethabi::Token::Address(self.address()),
            ethabi::Token::Uint(U256::from(self.expiration().timestamp())),
            ethabi::Token::Address(self.market()),
            ethabi::Token::Uint(self.nonce()),
        ];
        ethabi::encode(&abi_tokens)
    }

    fn reassemble_signed_message(&self) -> Vec<u8> {
        /* grab the byte slices for each of the three components of the final
         * signature message */
        let magic_prefix_bytes: &[u8] =
            &hex::decode(EIP712_MAGIC_PREFIX).unwrap();
        let domain_prefix_bytes: &[u8] = &hex::decode(DOMAIN_HASH).unwrap();

        /* hash the order component itself */
        let order_digest_bytes: &[u8] = {
            let mut hasher = Keccak256::new();
            hasher.update(DOMAIN_HASH);
            &hasher.finalize()
        };

        /* concatenate each of the three components together after appropriate
         * hashing */

        /* this mess only has to occur because (as of the time of writing),
         * `std::slice::Concat` is still Nightly-only -_-
         *
         * See: https://github.com/rust-lang/rust/issues/27747
         */
        let mut bytes_to_encode_vector: Vec<u8> = magic_prefix_bytes.to_vec();

        let final_bytes: &[u8] = {
            let mut domain_prefix_vector: Vec<u8> =
                domain_prefix_bytes.to_vec();
            let mut order_vector: Vec<u8> = order_digest_bytes.to_vec();

            domain_prefix_vector.append(&mut order_vector);
            bytes_to_encode_vector.append(&mut domain_prefix_vector);

            bytes_to_encode_vector.as_ref()
        };

        let _abi_tokens: Vec<ethabi::Token> = vec![
            ethabi::Token::FixedBytes(magic_prefix_bytes.to_vec()),
            ethabi::Token::FixedBytes(domain_prefix_bytes.to_vec()),
            ethabi::Token::FixedBytes(final_bytes.to_vec()),
        ];

        /* hash the final bytes */
        let mut hasher = Keccak256::new();
        hasher.update(bytes_to_encode_vector);
        let final_bytes: Vec<u8> = hasher.finalize().to_vec();

        final_bytes
    }

    /// Deserialises an order from its byte-level representation
    ///
    /// This function will perform validation on the input bytes to ensure that
    /// they are valid.
    #[allow(unused_variables)] /* TODO: remove when from_bytes is implemented */
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, OrderParseError> {
        unimplemented!() /* TODO: implement from_bytes */
    }

    /// Determines whether the order data matches the provided digital signature
    #[allow(unreachable_code)] /* TODO: remove before flight! */
    pub fn verify(&self) -> bool {
        return true; /* TODO: remove before flight! */

        let recovery_id: i32 = {
            let recovery_type: Recovery = match Recovery::from_raw_signature(
                self.to_bytes(),
                self.signed_data(),
            ) {
                Ok(t) => t,
                Err(_e) => return false,
            };
            match recovery_type.recovery_id() {
                Some(id) => id,
                None => return false,
            }
        };

        match recover(
            self.reassemble_signed_message().as_ref(),
            self.signed_data(),
            recovery_id,
        ) {
            Ok(signer_address) => signer_address == self.address,
            Err(_e) => false,
        }
    }
}
