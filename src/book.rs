//! Contains logic and type definitions for the order book itself and the
//! matching engine also
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    fmt::Display,
};

use chrono::{DateTime, Utc};
use ethereum_types::U256;
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
        (self.bids.len(), self.asks.len())
    }

    /// Returns whether the order book is currently crossed or not
    pub fn crossed(&self) -> bool {
        self.crossed
    }

    /// Returns the bid-ask spread of the book
    pub fn spread(&self) -> U256 {
        self.spread
    }

    /// Submits an order to the matching engine
    ///
    /// In the event the order cannot be (fully) matched, it will be stored
    /// in the order book for future matching.
    pub async fn submit(
        &mut self,
        mut order: Order,
        executioner_address: String,
    ) -> Result<(), BookError> {
        info!("Received {}", order);

        // load in book state
        let order_side = order.side;
        let order_price = order.price;
        let mut order_amount = order.amount;
        let bid_list = RefCell::new(&mut self.bids);
        let ask_list = RefCell::new(&mut self.asks);

        // get the best price and orders at this price
        let mut opposite = match order_side {
            OrderSide::Bid => {
                let lowest_price_ask = ask_list.borrow_mut().pop_first();
                lowest_price_ask
            }
            OrderSide::Ask => {
                let highest_price_bid = bid_list.borrow_mut().pop_last();
                highest_price_bid
            }
        };

        // no orders available to match, place order on the books
        if opposite.is_none() {
            // Nothing on the opposite side.
            info!("Adding {} due to lack of counterparty volume", order);
            return self.add_order(order); // Store the order and we are done.
        }

        let (mut price, mut orders_queue) = opposite.unwrap();

        // done if the price crosses. Changes if the order is a bid or ask
        let mut done = if order_side == OrderSide::Bid {
            order_price < price
        } else {
            order_price > price
        };

        // match until either the order is fully matched, or all valid orders to match with are done.
        while !done {
            // iterate over orders at the current "best" price
            while let Some(mut matching_order) = orders_queue.pop_front() {
                // compute new amounts for our orders using temp variables
                let match_to_submit = matching_order.clone();
                let mut matching_amount = matching_order.amount;
                matching_order.amount =
                    matching_amount.saturating_sub(order_amount);
                order_amount = order_amount.saturating_sub(matching_amount);
                matching_amount = matching_order.amount;

                // forward to the contracts
                info!("Forwarding ({},{})", order, match_to_submit);

                /* push to contract */
                match rpc::send_matched_orders(
                    order.clone(),
                    match_to_submit,
                    executioner_address.clone(),
                )
                .await
                {
                    Ok(t) => {
                        info!("See https://etherscan.io/tx/{}", t);
                    }
                    Err(e) => {
                        warn!("Failed to forward matched orders! {}", e);
                        // todo: how should we handle this when no executioner is running
                        // eg in tests?
                        //return Err(BookError::Web3Error);
                    }
                };

                info!("Executed {}", matching_order);

                // stop if the order_amount is filled.
                if order_amount.is_zero() {
                    if !matching_amount.is_zero() {
                        // todo: push back seems like it should be put to the front?
                        // Put the last popped out order back in the queue.
                        orders_queue.push_back(matching_order);
                    }
                    break;
                }

                // run out of orders at this price however we still have position
                // to fill (order amount != 0)
                if orders_queue.is_empty() {
                    break;
                }
            }

            // Check if we are done. If not move to next price level
            if order_amount.is_zero() {
                break;
            } else {
                opposite = match order_side {
                    OrderSide::Bid => {
                        let lowest_price_ask =
                            ask_list.borrow_mut().pop_first();
                        lowest_price_ask
                    }
                    OrderSide::Ask => {
                        let highest_price_bid =
                            bid_list.borrow_mut().pop_last();
                        highest_price_bid
                    }
                };
                if opposite.is_none() {
                    // Nothing on the opposite side.
                    info!(
                        "Adding {} due to lack of counterparty volume",
                        order
                    );
                    return self.add_order(order); // Store the order and we are done.
                }

                // revaluate if we are done
                (price, orders_queue) = opposite.unwrap();
                done = if order_side == OrderSide::Bid {
                    order_price < price
                } else {
                    order_price > price
                };
            }
        }

        // put any outstanding orders back into their rightful order queue
        if !orders_queue.is_empty() {
            match order_side {
                OrderSide::Bid => {
                    let mut asks = ask_list.borrow_mut();
                    asks.insert(price, orders_queue);
                }
                OrderSide::Ask => {
                    let mut bids = bid_list.borrow_mut();
                    bids.insert(price, orders_queue);
                }
            }
        }

        // if the order still isn't filled, its not possible.
        // place it on the books
        if !order_amount.is_zero() {
            // Order still has units but could not be filled
            *order.amount_mut() = order_amount;
            let _ = self.add_order(order);
        }

        Ok(())
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
