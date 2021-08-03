//! Contains logic for interacting with the OME's state
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use web3::types::Address;

use crate::book::Book;

/// Represents the entire state of the OME
#[derive(Clone, PartialEq, Eq, Default, Debug, Serialize, Deserialize)]
pub struct OmeState {
    books: HashMap<Address, Book>,
}

impl OmeState {
    /// Constructor for the `OmeState` type
    pub fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }

    /// Returns a reference to the mapping from tickers to `Book` types
    /// themselves.
    pub fn books(&self) -> &HashMap<Address, Book> {
        &self.books
    }

    /// Returns a reference to a specific order book
    pub fn book(&self, market: Address) -> Option<&Book> {
        self.books.get(&market)
    }

    /// Returns a mutable reference to a specific order book
    pub fn book_mut(&mut self, market: Address) -> Option<&mut Book> {
        self.books.get_mut(&market)
    }

    /// Add a new order book to the OME
    pub fn add_book(&mut self, book: Book) {
        self.books.insert(*book.market(), book);
    }

    /// Remove an order book from the OME
    pub fn remove_book(&mut self, market: Address) -> Option<Book> {
        self.books.remove(&market)
    }
}
