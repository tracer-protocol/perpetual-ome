use chrono::Utc;
use ethereum_types::{Address, U256};

use crate::book::{Book, BookError};
use crate::order::{Order, OrderSide};

pub const TEST_RPC_ADDRESS: &str = "http://localhost:3000";

async fn submit_orders(
    market: Address,
    data: Vec<(Address, OrderSide, u64, u64)>,
) -> Book {
    /* build orders from supplied parameters */
    let orders: Vec<Order> = data
        .iter()
        .map(|(addr, side, price, qty)| {
            Order::new(
                *addr,
                market,
                *side,
                (*price).into(),
                (*qty).into(),
                Utc::now(),
                vec![],
            )
        })
        .collect();

    let mut book: Book = Book::new(market);

    /* apply each order to the book (sadly we can't `map` here due to our blocking requirement) */
    for order in orders {
        book.submit(order.clone(), TEST_RPC_ADDRESS.to_string())
            .await
            .expect("Failed to submit order to book")
    }

    book
}

async fn setup() -> Book {
    let market: Address = Address::zero();

    /* placeholders for trader addresses (saves us writing real Ethereum addresses) */
    let traders: Vec<Address> =
        (0..10).map(|x| Address::from_low_u64_be(x)).collect();

    let asks: Vec<(Address, OrderSide, u64, u64)> = vec![
        (traders[0], OrderSide::Ask, 100, 10),
        (traders[1], OrderSide::Ask, 99, 2),
        (traders[2], OrderSide::Ask, 98, 35),
        (traders[3], OrderSide::Ask, 97, 15),
        (traders[4], OrderSide::Ask, 96, 5),
    ];

    let bids: Vec<(Address, OrderSide, u64, u64)> = vec![
        (traders[5], OrderSide::Bid, 95, 10),
        (traders[6], OrderSide::Bid, 94, 20),
        (traders[7], OrderSide::Bid, 93, 5),
        (traders[8], OrderSide::Bid, 92, 10),
        (traders[9], OrderSide::Bid, 91, 15),
    ];

    let orders: Vec<(Address, OrderSide, u64, u64)> =
        bids.iter().cloned().chain(asks.iter().cloned()).collect();

    submit_orders(market, orders).await
}

#[tokio::test]
pub async fn test_new_book() {
    let market: Address = Address::zero();
    let book = Book::new(market);

    assert_eq!(book.market(), &market);
    assert_eq!(book.depth(), (0, 0)); // Asserts that there are no orders in the book.
}

#[tokio::test]
pub async fn test_book_depth() {
    let book = setup().await;

    let (bid_length, ask_length) = book.depth();

    assert_eq!(bid_length, 5);
    assert_eq!(ask_length, 5);
}

#[tokio::test]
pub async fn test_simple_buy() {
    let mut book = setup().await;
    let bid = Order::new(
        Address::from_low_u64_be(3),
        Address::zero(),
        OrderSide::Bid,
        U256::from_dec_str(&"96").unwrap(),
        U256::from_dec_str(&"5").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    assert_eq!(bid_length, 5);
    assert_eq!(ask_length, 4);
}

#[tokio::test]
pub async fn test_simple_buy_partially_filled() {
    let mut book = setup().await;

    let market_address = Address::zero();
    let bid = Order::new(
        Address::from_low_u64_be(3),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"96").unwrap(),
        U256::from_dec_str(&"3").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    // Ensure the depths are correct
    assert_eq!(bid_length, 5);
    assert_eq!(ask_length, 5);
}

#[tokio::test]
pub async fn test_simple_sell() {
    let mut book = setup().await;
    let ask = Order::new(
        Address::from_low_u64_be(3),
        Address::zero(),
        OrderSide::Ask,
        U256::from_dec_str(&"95").unwrap(),
        U256::from_dec_str(&"10").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(ask, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    assert_eq!(bid_length, 4);
    assert_eq!(ask_length, 5);
}

#[tokio::test]
pub async fn test_simple_sell_partially_filled() {
    let mut book = setup().await;

    let market_address = Address::zero();
    let bid = Order::new(
        Address::from_low_u64_be(3),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"95").unwrap(),
        U256::from_dec_str(&"1").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    // Ensure the depths are correct
    assert_eq!(bid_length, 5);
    assert_eq!(ask_length, 5);
}

#[tokio::test]
pub async fn test_deep_buy() {
    let mut book = setup().await;
    let market_address = Address::zero();
    let bid = Order::new(
        Address::from_low_u64_be(10),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"99").unwrap(),
        U256::from_dec_str(&"42").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    // Ensure the depths are correct
    assert_eq!(bid_length, 5);
    assert_eq!(ask_length, 3);
}

#[tokio::test]
pub async fn test_no_self_matching() {
    let mut book = setup().await;
    let market: Address = Address::zero();

    let bid: Order = Order::new(
        Address::from_low_u64_be(1), /* previously placed a ask */
        market,
        OrderSide::Bid,
        97.into(),
        15.into(),
        Utc::now(),
        vec![],
    );

    let actual_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_depth, ask_depth) = book.depth();

    assert_eq!(actual_res, Ok(()));
    assert_eq!(bid_depth, 5);
    assert_eq!(ask_depth, 4);
}

#[tokio::test]
pub async fn test_no_self_matching_when_last_order() {
    let market: Address = Address::zero();
    let mut book = Book::new(market);

    let ask: Order = Order::new(
        Address::from_low_u64_be(1),
        market,
        OrderSide::Ask,
        97.into(),
        15.into(),
        Utc::now(),
        vec![],
    );

    let bid: Order = Order::new(
        Address::from_low_u64_be(1), /* previously placed a ask */
        market,
        OrderSide::Bid,
        97.into(),
        15.into(),
        Utc::now(),
        vec![],
    );

    book.submit(ask, TEST_RPC_ADDRESS.to_string())
        .await
        .unwrap();

    let actual_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_depth, ask_depth) = book.depth();

    assert_eq!(actual_res, Ok(()));
    assert_eq!(bid_depth, 1);
    assert_eq!(ask_depth, 1);
}

#[tokio::test]
pub async fn test_deep_buy_with_limit() {
    let mut book = setup().await;
    let market_address = Address::zero();
    let bid = Order::new(
        Address::from_low_u64_be(10),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"97").unwrap(),
        U256::from_dec_str(&"42").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(bid, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    assert_eq!(bid_length, 6); // There should be one more bid with 22 units at 97.
    assert_eq!(ask_length, 3);
}

#[tokio::test]
pub async fn test_deep_sell() {
    let mut book = setup().await;
    let market_address = Address::zero();
    let ask = Order::new(
        Address::from_low_u64_be(10),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"94").unwrap(),
        U256::from_dec_str(&"20").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(ask, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    // Ensure the depths are correct
    assert_eq!(bid_length, 4);
    assert_eq!(ask_length, 5);
}

#[tokio::test]
pub async fn test_deep_sell_with_limit() {
    let mut book = setup().await;
    let market_address = Address::zero();
    let ask = Order::new(
        Address::from_low_u64_be(33),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"94").unwrap(),
        U256::from_dec_str(&"35").unwrap(),
        Utc::now(),
        vec![],
    );

    let submit_res: Result<(), BookError> =
        book.submit(ask, TEST_RPC_ADDRESS.to_string()).await;

    let (bid_length, ask_length) = book.depth();

    assert!(
        submit_res.is_ok() || submit_res.contains_err(&BookError::Web3Error)
    );

    assert_eq!(bid_length, 3);
    assert_eq!(ask_length, 6); // There should be one more ask with 5 units at 94.
}
