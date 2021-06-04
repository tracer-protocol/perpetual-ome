//! Contains logic and type definitions for the order book itself and the
//! matching engine also
use std::{
    cmp::Ordering,
    collections::{BTreeMap, VecDeque},
    fmt::Display,
};

use chrono::{DateTime, Utc};
use ethereum_types::U256;
use itertools::Either;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use web3::types::Address;

use crate::order::{Order, OrderId, OrderSide};
use crate::rpc;
use crate::util::{from_hex_de, from_hex_se};

/// Represents an order book for a particular Tracer market
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Book {
    market: Address, /* the address of the Tracer market */
    pub bids: BTreeMap<U256, VecDeque<Order>>, /* buy-side */
    pub asks: BTreeMap<U256, VecDeque<Order>>, /* sell-side */
    #[serde(
        serialize_with = "from_hex_se",
        deserialize_with = "from_hex_de",
        rename = "LTP"
    )]
    ltp: U256, /* last traded price */
    depth: (usize, usize), /* depth  */
    crossed: bool,   /* is book crossed? */
    #[serde(serialize_with = "from_hex_se", deserialize_with = "from_hex_de")]
    spread: U256, /* bid-ask spread */
}

#[derive(
    Clone, Copy, Debug, Display, Error, Serialize, Deserialize, PartialEq, Eq,
)]
pub enum BookError {
    Web3Error,
}

impl From<web3::Error> for BookError {
    fn from(_error: web3::Error) -> Self {
        BookError::Web3Error
    }
}

impl From<ethabi::Error> for BookError {
    fn from(_error: ethabi::Error) -> Self {
        BookError::Web3Error
    }
}

impl Book {
    /// Constructor for the `Book` type
    ///
    /// Takes the address of the underlying Tracer contract as its sole
    /// argument, then initialises both sides of the book to be empty.
    pub fn new(market: Address) -> Self {
        Self {
            market,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            ltp: Default::default(),
            depth: (0, 0),
            crossed: false,
            spread: Default::default(),
        }
    }

    /// Returns the ticker of this market
    pub fn market(&self) -> &Address {
        &self.market
    }

    /// Returns a reference to the order matching the provided order ID
    pub fn order(&self, id: OrderId) -> Option<&Order> {
        /* search bids */
        for (_, curr_level) in self.bids.iter() {
            for curr_order in curr_level.iter() {
                if curr_order.id == id {
                    return Some(curr_order);
                }
            }
        }

        /* search asks */
        for (_, curr_level) in self.asks.iter() {
            for curr_order in curr_level.iter() {
                if curr_order.id == id {
                    return Some(curr_order);
                }
            }
        }

        None
    }

    /// Returns a mutable reference to the order matching the provided order ID
    pub fn order_mut(&mut self, id: OrderId) -> Option<&mut Order> {
        /* search bids */
        for (_, curr_level) in self.bids.iter_mut() {
            for curr_order in curr_level.iter_mut() {
                if curr_order.id == id {
                    return Some(curr_order);
                }
            }
        }

        /* search asks */
        for (_, curr_level) in self.asks.iter_mut() {
            for curr_order in curr_level.iter_mut() {
                if curr_order.id == id {
                    return Some(curr_order);
                }
            }
        }

        None
    }

    /// Returns the last traded price of the order book
    pub fn ltp(&self) -> U256 {
        self.ltp
    }

    /// Returns a pair (2-tuple) containing the depths of each side of the book
    pub fn depth(&self) -> (usize, usize) {
        (
            self.bids
                .values()
                .flatten()
                .filter(|order| !order.amount_left.is_zero())
                .count(),
            self.asks
                .values()
                .flatten()
                .filter(|order| !order.amount_left.is_zero())
                .count(),
        )
    }

    /// Returns whether the order book is currently crossed or not
    pub fn crossed(&self) -> bool {
        self.crossed
    }

    /// Returns the bid-ask spread of the book
    pub fn spread(&self) -> U256 {
        self.spread
    }

    pub fn top(&self) -> (Option<U256>, Option<U256>) {
        (
            self.bids.last_key_value().map(|t| *t.0),
            self.asks.first_key_value().map(|t| *t.0),
        )
    }

    fn price_viable(
        opposite: U256,
        incoming: U256,
        incoming_side: OrderSide,
    ) -> bool {
        match incoming_side {
            OrderSide::Bid => opposite <= incoming,
            OrderSide::Ask => opposite >= incoming,
        }
    }

    #[allow(unused_must_use)]
    async fn r#match(
        &mut self,
        mut order: Order,
        executioner_address: String,
        opposing_top: Option<U256>,
    ) -> Result<(), BookError> {
        let opposing_side: &mut BTreeMap<U256, VecDeque<Order>> =
            match order.side {
                OrderSide::Bid => &mut self.asks,
                OrderSide::Ask => &mut self.bids,
            };
        let mut running_total: U256 = order.amount_left;
        let mut done: bool = false;

        /* if we haven't crossed the spread, we're not going to match */
        if opposing_top.is_none()
            || !Book::price_viable(
                opposing_top.unwrap(),
                order.price,
                order.side,
            )
        {
            return self.add_order(order);
        }

        let opposing_side_iterator = match order.side {
            OrderSide::Bid => Either::Left(opposing_side.iter_mut()),
            OrderSide::Ask => Either::Right(opposing_side.iter_mut().rev()),
        };

        for (price, opposites) in opposing_side_iterator {
            /* if we've run out of viable prices or we're done, halt */
            if done || !Book::price_viable(*price, order.price, order.side) {
                break;
            }

            for opposite in opposites {
                /* no self-trading allowed */
                if opposite.user == order.user {
                    continue;
                }

                /* determine how much to match */
                let amount: U256 =
                    match opposite.amount_left.cmp(&order.amount_left) {
                        Ordering::Greater => order.amount_left,
                        _ => opposite.amount_left,
                    };

                /* match */
                order = Book::fill(order, amount);
                *opposite = Book::fill(opposite.clone(), amount);

                self.ltp = *price;

                rpc::send_matched_orders(
                    order.clone(),
                    opposite.clone(),
                    executioner_address.clone(),
                )
                .await;

                if opposite.amount_left.is_zero() { /* TODO: delete ask order */
                }

                dbg!(&running_total, &amount);
                running_total -= amount;

                /* check if we've totally matched our incoming order */
                if running_total.is_zero() {
                    done = true;
                    break;
                }
            }
        }

        /* if our incoming order has any volume left, add it to the book */
        if running_total > U256::zero() {
            self.add_order(order)
        } else {
            Ok(())
        }
    }

    fn fill(order: Order, amount: U256) -> Order {
        match amount.cmp(&order.amount_left) {
            Ordering::Greater => order,
            _ => Order {
                id: order.id,
                user: order.user,
                target_tracer: order.target_tracer,
                side: order.side,
                price: order.price,
                amount,
                amount_left: order.amount_left - amount,
                expiration: order.expiration,
                created: order.created,
                signed_data: order.signed_data,
            },
        }
    }

    /// Submits an order to the matching engine
    ///
    /// In the event the order cannot be (fully) matched, it will be stored
    /// in the order book for future matching.
    pub async fn submit(
        &mut self,
        order: Order,
        executioner_address: String,
    ) -> Result<(), BookError> {
        info!("Received {}", order);

        match order.side {
            OrderSide::Bid => {
                self.r#match(order, executioner_address, self.top().1).await
            }
            OrderSide::Ask => {
                self.r#match(order, executioner_address, self.top().0).await
            }
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn add_order(&mut self, order: Order) -> Result<(), BookError> {
        let tmp_order: Order = order.clone();
        let order_side = order.side;
        let order_price = order.price;
        let orders = VecDeque::new();

        match order_side {
            OrderSide::Bid => {
                self.bids
                    .entry(order_price)
                    .or_insert(orders)
                    .push_back(order);
            }
            OrderSide::Ask => {
                self.asks
                    .entry(order_price)
                    .or_insert(orders)
                    .push_back(order);
            }
        }

        info!("Added {}", tmp_order);

        // TODO: Not sure if we will ever run this far.
        Ok(())
    }

    /*******************HELPER FUNCTIONS FOR TESTING END************************/

    /// Cancels the open order currently in the order book with the matching ID
    ///
    /// # Returns #
    ///
    /// Returns `Ok(Some(dt))` upon success, where `dt` is a `DateTime<Utc>`
    /// type representing the time of successful cancellation of the order.
    ///
    /// Returns `Ok(None)` if there is no such order currently in the book.
    ///
    /// Returns a `BookError` if there is an error condition
    #[allow(unused_variables)] /* TODO: remove when cancel is implemented */
    pub fn cancel(
        &mut self,
        order_id: OrderId,
    ) -> Result<Option<DateTime<Utc>>, BookError> {
        for (_, orders) in self.bids.iter_mut() {
            for (index, order) in orders.iter_mut().enumerate() {
                if order.id == order_id {
                    info!("Cancelled {}", order.clone());
                    orders.remove(index);
                    return Ok(Some(Utc::now()));
                }
            }
        }

        for (_, orders) in self.asks.iter_mut() {
            for (index, order) in orders.iter_mut().enumerate() {
                if order.id == order_id {
                    info!("Cancelled {}", order.clone());
                    orders.remove(index);
                    return Ok(Some(Utc::now()));
                }
            }
        }

        Ok(None)
    }

    /// Updates internal metadata of the order book
    ///
    /// Should be called *after successful* mutation of order book state.
    #[allow(dead_code)]
    fn update(&mut self) {
        self.depth = (self.bids.len(), self.asks.len());
        /* TODO: check for crossed state */
    }
}
