use std::time::Duration;

use flutter_rust_bridge::frb;
use reqwest::blocking::Client;
use reqwest::StatusCode;
use serde::Deserialize;

use super::super::error::LwkError;
use super::config::{indexer_base_url, with_lending_config};
use super::types::{
    parse_offer_details_json, LendingOfferDetails, LendingOfferListQuery, LendingOfferListResponse,
    LendingOffersOverview,
};

const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// Read-only HTTP client for the simplicity-lending indexer.
pub struct LendingIndexer {}

impl LendingIndexer {
    pub fn new() -> Self {
        Self {}
    }

    /// `GET /offers` — paginated offer list.
    #[frb]
    pub fn list_offers(
        &self,
        query: LendingOfferListQuery,
    ) -> anyhow::Result<LendingOfferListResponse, LwkError> {
        super::super::ensure_crypto_provider();
        with_lending_config(|config| {
            let base_url = indexer_base_url(config)?;
            let url = format!("{base_url}/offers");
            get_json(&url, Some(query.to_query_pairs()))
        })
    }

    /// `GET /offers/{id}` — full offer details.
    #[frb]
    pub fn get_details(&self, id: String) -> anyhow::Result<LendingOfferDetails, LwkError> {
        super::super::ensure_crypto_provider();
        with_lending_config(|config| {
            let base_url = indexer_base_url(config)?;
            let url = format!("{base_url}/offers/{id}");
            let body = get_text(&url, None)?;
            parse_offer_details_json(&body).map_err(|err| LwkError::from(err.to_string()))
        })
    }

    /// `GET /offers/overview` — protocol-wide active loan totals.
    #[frb]
    pub fn get_overview(&self) -> anyhow::Result<LendingOffersOverview, LwkError> {
        super::super::ensure_crypto_provider();
        with_lending_config(|config| {
            let base_url = indexer_base_url(config)?;
            let url = format!("{base_url}/offers/overview");
            get_json(&url, None)
        })
    }

    /// `GET /offers/by-script` — offer IDs with an unspent participant UTXO.
    #[frb]
    pub fn get_ids_by_script(
        &self,
        script_pubkey: String,
    ) -> anyhow::Result<Vec<String>, LwkError> {
        super::super::ensure_crypto_provider();
        with_lending_config(|config| {
            let base_url = indexer_base_url(config)?;
            let url = format!("{base_url}/offers/by-script");
            let pairs = vec![("script_pubkey".to_string(), script_pubkey)];
            get_json(&url, Some(pairs))
        })
    }
}

fn http_client() -> Result<Client, LwkError> {
    Client::builder()
        .timeout(HTTP_REQUEST_TIMEOUT)
        .build()
        .map_err(|err| LwkError::from(err.to_string()))
}

fn get_text(url: &str, query: Option<Vec<(String, String)>>) -> Result<String, LwkError> {
    let client = http_client()?;
    let mut request = client.get(url);
    if let Some(pairs) = query {
        request = request.query(&pairs);
    }

    let response = request
        .send()
        .map_err(|err| LwkError::from(err.to_string()))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|err| LwkError::from(err.to_string()))?;

    if status.is_success() {
        return Ok(body);
    }

    Err(indexer_http_error(status, &body))
}

fn get_json<T>(url: &str, query: Option<Vec<(String, String)>>) -> Result<T, LwkError>
where
    T: for<'de> Deserialize<'de>,
{
    let body = get_text(url, query)?;
    serde_json::from_str(&body).map_err(|err| LwkError::from(err.to_string()))
}

fn indexer_http_error(status: StatusCode, body: &str) -> LwkError {
    let msg = if body.is_empty() {
        format!("Indexer request failed with status {status}")
    } else {
        format!("Indexer request failed with status {status}: {body}")
    };
    LwkError::from(msg)
}
