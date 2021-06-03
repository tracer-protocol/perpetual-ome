#[cfg(test)]
mod order_tests {
    use chrono::Utc;
    use web3::types::{Address, U256};

    use crate::order::{Order, OrderSide};

    #[test]
    pub fn basic_order_info() {
        let address = Address::from_low_u64_be(1);
        let market_address = Address::zero();
        let price = U256::from_dec_str(&"5").unwrap();
        let amount = U256::from_dec_str(&"10").unwrap();

        let order = Order::new(
            address,
            market_address,
            OrderSide::Bid,
            price,
            amount,
            Utc::now(),
            vec![],
        );

        assert_eq!(order.user, address);
        assert_eq!(order.target_tracer, market_address);
        assert_eq!(order.side, OrderSide::Bid);
        assert_eq!(order.price, price);
        assert_eq!(order.amount, amount);
        assert!(order.signed_data.is_empty());
    }
}

#[cfg(test)]
mod state_tests {
    use web3::types::Address;

    use crate::book::Book;
    use crate::OmeState;

    #[test]
    pub fn new_state_should_be_empty() {
        let state = OmeState::new();

        assert!(state.books().is_empty());
    }

    #[test]
    pub fn add_book() {
        let mut state = OmeState::new();
        let market: Address = Address::zero();
        let book = Book::new(market);

        state.add_book(book);

        assert!(state.books().len() == 1);
        assert!(state.book(market).is_some());
    }

    #[test]
    pub fn remove_book() {
        let mut state = OmeState::new();
        let market: Address = Address::zero();
        let book = Book::new(market);
        state.add_book(book);

        let market2 = Address::repeat_byte(1);
        let book2 = Book::new(market2);
        state.add_book(book2);

        state.remove_book(market2);
        assert!(state.books().len() == 1);

        state.remove_book(market);
        assert!(state.books().is_empty());
    }
}
