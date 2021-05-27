use chrono::Utc;
use ethereum_types::{Address, U256};

use crate::book::{Book, BookError};
use crate::order::{Order, OrderSide};

pub const TEST_RPC_ADDRESS: &str = "http://localhost:3000";

async fn setup() -> Book {
    let market: Address = Address::zero();
    let mut book = Book::new(market);

    let market_address = Address::zero();
    let ask = Order::new(
        Address::from_low_u64_be(1),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"100").unwrap(),
        U256::from_dec_str(&"10").unwrap(),
        Utc::now(),
        vec![],
    );

    let ask1 = Order::new(
        Address::from_low_u64_be(2),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"99").unwrap(),
        U256::from_dec_str(&"2").unwrap(),
        Utc::now(),
        vec![],
    );

    let ask2 = Order::new(
        Address::from_low_u64_be(3),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"98").unwrap(),
        U256::from_dec_str(&"35").unwrap(),
        Utc::now(),
        vec![],
    );

    let ask3 = Order::new(
        Address::from_low_u64_be(4),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"97").unwrap(),
        U256::from_dec_str(&"15").unwrap(),
        Utc::now(),
        vec![],
    );

    let ask4 = Order::new(
        Address::from_low_u64_be(5),
        market_address,
        OrderSide::Ask,
        U256::from_dec_str(&"96").unwrap(),
        U256::from_dec_str(&"5").unwrap(),
        Utc::now(),
        vec![],
    );

    book.submit(ask, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the ask order 1.");
    book.submit(ask1, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the ask order 2.");
    book.submit(ask2, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the ask order 3.");
    book.submit(ask3, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the ask order 4.");
    book.submit(ask4, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the ask order 5.");

    let bid = Order::new(
        Address::from_low_u64_be(1),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"95").unwrap(),
        U256::from_dec_str(&"10").unwrap(),
        Utc::now(),
        vec![],
    );

    let bid1 = Order::new(
        Address::from_low_u64_be(2),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"94").unwrap(),
        U256::from_dec_str(&"20").unwrap(),
        Utc::now(),
        vec![],
    );

    let bid2 = Order::new(
        Address::from_low_u64_be(3),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"93").unwrap(),
        U256::from_dec_str(&"5").unwrap(),
        Utc::now(),
        vec![],
    );

    let bid3 = Order::new(
        Address::from_low_u64_be(4),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"92").unwrap(),
        U256::from_dec_str(&"10").unwrap(),
        Utc::now(),
        vec![],
    );

    let bid4 = Order::new(
        Address::from_low_u64_be(5),
        market_address,
        OrderSide::Bid,
        U256::from_dec_str(&"91").unwrap(),
        U256::from_dec_str(&"15").unwrap(),
        Utc::now(),
        vec![],
    );

    book.submit(bid, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the bid order 1.");
    book.submit(bid1, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the bid order 2.");
    book.submit(bid2, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the bid order 3.");
    book.submit(bid3, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the bid order 4.");
    book.submit(bid4, TEST_RPC_ADDRESS.to_string())
        .await
        .expect("Error submitting the bid order 5.");

    book
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
        Address::from_low_u64_be(3),
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
pub async fn test_deep_buy_with_limit() {
    let mut book = setup().await;
    let market_address = Address::zero();
    let bid = Order::new(
        Address::from_low_u64_be(3),
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
