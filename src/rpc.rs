use std::fmt::Display;

use reqwest::{header, Client, Response};
use serde::{Deserialize, Serialize};

use crate::book::ExternalBook;

#[derive(Display, Debug)]
pub enum RpcError {
    HttpError,
    ContractError,
    InvalidResponse,
}

impl From<reqwest::Error> for RpcError {
    fn from(_value: reqwest::Error) -> Self {
        Self::HttpError
    }
}

impl From<rustc_hex::FromHexError> for RpcError {
    fn from(_value: rustc_hex::FromHexError) -> Self {
        Self::InvalidResponse
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KnownMarketsResponse {
    message: String,
    data: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExternalBookResponse {
    message: String,
    data: ExternalBook,
}

pub async fn get_known_markets(address: &str) -> Result<Vec<String>, RpcError> {
    let endpoint: String = address.to_string();
    let client: Client = Client::new();

    info!("Fetching known markets from {}...", endpoint);

    let response: Response = match client
        .get(endpoint.clone())
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await
    {
        Ok(t) => t,
        Err(e) => return Err(e.into()),
    };

    let markets: KnownMarketsResponse = response.json().await?;

    info!("{} said {:?}", endpoint, markets);

    Ok(markets.data)
}

pub async fn get_external_book(
    address: &str,
    market_id: String,
) -> Result<ExternalBook, RpcError> {
    let endpoint: String = address.to_string();
    let client: Client = Client::new();

    info!("Fetching external book from {}...", endpoint);

    let response: Response = match client
        .get(endpoint.clone() + &market_id)
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await
    {
        Ok(t) => t,
        Err(e) => return Err(e.into()),
    };

    let book: ExternalBookResponse = response.json().await?;

    info!("{} said {:?}", endpoint, book);

    Ok(book.data)
}
