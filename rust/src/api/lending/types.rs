use serde::{Deserialize, Serialize};

/// Offer lifecycle status from the indexer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LendingOfferStatus {
    Pending,
    Active,
    Repaid,
    Liquidated,
    Cancelled,
    Claimed,
}

impl LendingOfferStatus {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Repaid => "repaid",
            Self::Liquidated => "liquidated",
            Self::Cancelled => "cancelled",
            Self::Claimed => "claimed",
        }
    }
}

/// Participant role in an offer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LendingParticipantType {
    Borrower,
    Lender,
}

/// Offer UTXO role tracked by the indexer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LendingUtxoType {
    PendingOffer,
    ActiveOffer,
    BorrowerPrincipal,
    Cancellation,
    Repayment,
    Liquidation,
    Claim,
}

/// Sort field for offer list queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LendingOfferSortBy {
    #[default]
    CreatedAtHeight,
    CollateralAmount,
    PrincipalAmount,
    InterestRate,
    LoanExpirationHeight,
}

impl LendingOfferSortBy {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::CreatedAtHeight => "created_at_height",
            Self::CollateralAmount => "collateral_amount",
            Self::PrincipalAmount => "principal_amount",
            Self::InterestRate => "interest_rate",
            Self::LoanExpirationHeight => "loan_expiration_height",
        }
    }
}

/// Sort direction for offer list queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LendingSortDir {
    #[default]
    Desc,
    Asc,
}

impl LendingSortDir {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Desc => "desc",
            Self::Asc => "asc",
        }
    }
}

/// Query parameters for `GET /offers`.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct LendingOfferListQuery {
    pub status: Option<Vec<LendingOfferStatus>>,
    pub collateral_asset: Option<String>,
    pub principal_asset: Option<String>,
    pub factory_id: Option<String>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub sort_by: LendingOfferSortBy,
    pub sort_dir: LendingSortDir,
}

impl LendingOfferListQuery {
    pub(crate) fn to_query_pairs(&self) -> Vec<(String, String)> {
        let mut pairs = Vec::new();

        if let Some(statuses) = &self.status {
            if !statuses.is_empty() {
                pairs.push((
                    "status".to_string(),
                    statuses
                        .iter()
                        .map(|status| status.as_str())
                        .collect::<Vec<_>>()
                        .join(","),
                ));
            }
        }
        if let Some(collateral_asset) = &self.collateral_asset {
            pairs.push(("collateral_asset".to_string(), collateral_asset.clone()));
        }
        if let Some(principal_asset) = &self.principal_asset {
            pairs.push(("principal_asset".to_string(), principal_asset.clone()));
        }
        if let Some(factory_id) = &self.factory_id {
            pairs.push(("factory_id".to_string(), factory_id.clone()));
        }
        if let Some(limit) = self.limit {
            pairs.push(("limit".to_string(), limit.to_string()));
        }
        if let Some(offset) = self.offset {
            pairs.push(("offset".to_string(), offset.to_string()));
        }
        pairs.push(("sort_by".to_string(), self.sort_by.as_str().to_string()));
        pairs.push(("sort_dir".to_string(), self.sort_dir.as_str().to_string()));

        pairs
    }
}

/// Asset amount pair from indexer overview responses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingAssetAmount {
    pub asset: String,
    pub amount: String,
}

/// Short participant entry in offer list items.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingParticipantShort {
    pub participant_type: LendingParticipantType,
    pub script_pubkey: String,
}

/// UTXO outpoint reference in offer list items.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingOfferUtxoOutpointShort {
    pub txid: String,
    pub vout: u32,
}

/// Short offer list item from `GET /offers`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingOfferListItem {
    pub id: String,
    pub issuance_factory_id: String,
    pub status: LendingOfferStatus,
    pub collateral_asset: String,
    pub principal_asset: String,
    pub collateral_amount: String,
    pub principal_amount: String,
    pub interest_rate: u32,
    pub loan_expiration_height: u32,
    pub created_at_height: u64,
    pub created_at_txid: String,
    #[serde(default)]
    pub participants: Vec<LendingParticipantShort>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub borrower_principal_utxo: Option<LendingOfferUtxoOutpointShort>,
}

/// Paginated offer list response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingOfferListResponse {
    pub items: Vec<LendingOfferListItem>,
    pub total: u64,
    pub limit: u64,
    pub offset: u64,
}

/// Protocol-wide active loan totals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingOffersOverview {
    pub collateral_locked: Vec<LendingAssetAmount>,
    pub active_loan_principal: Vec<LendingAssetAmount>,
    pub active_loans_count: u64,
}

/// Full participant UTXO from offer details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingParticipantDto {
    pub offer_id: String,
    pub participant_type: LendingParticipantType,
    pub script_pubkey: String,
    pub txid: String,
    pub vout: u32,
    pub created_at_height: u64,
    pub spent_txid: Option<String>,
    pub spent_at_height: Option<u64>,
}

/// Offer UTXO from offer details.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingOfferUtxoDto {
    pub offer_id: String,
    pub txid: String,
    pub vout: u32,
    pub utxo_type: LendingUtxoType,
    pub created_at_height: u64,
    pub spent_txid: Option<String>,
    pub spent_at_height: Option<u64>,
}

/// Full offer details from `GET /offers/{id}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LendingOfferDetails {
    pub id: String,
    pub issuance_factory_id: String,
    pub status: LendingOfferStatus,
    pub collateral_asset: String,
    pub principal_asset: String,
    pub collateral_amount: String,
    pub principal_amount: String,
    pub interest_rate: u32,
    pub loan_expiration_height: u32,
    pub created_at_height: u64,
    pub created_at_txid: String,
    pub borrower_nft_asset: String,
    pub lender_nft_asset: String,
    pub protocol_fee_keeper_asset: String,
    #[serde(default)]
    pub participants: Vec<LendingParticipantDto>,
    #[serde(default)]
    pub utxos: Vec<LendingOfferUtxoDto>,
}

pub(crate) fn parse_offer_details_json(body: &str) -> Result<LendingOfferDetails, serde_json::Error> {
    serde_json::from_str(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST_RESPONSE_JSON: &str = r#"{
        "items": [{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "issuance_factory_id": "660e8400-e29b-41d4-a716-446655440001",
            "status": "pending",
            "collateral_asset": "030201",
            "principal_asset": "060504",
            "collateral_amount": "1000",
            "principal_amount": "500",
            "interest_rate": 120,
            "loan_expiration_height": 12345,
            "created_at_height": 100,
            "created_at_txid": "ccbbaa",
            "participants": [{
                "participant_type": "borrower",
                "script_pubkey": "51ac"
            }],
            "borrower_principal_utxo": {
                "txid": "aabb",
                "vout": 1
            }
        }],
        "total": 1,
        "limit": 50,
        "offset": 0
    }"#;

    const DETAILS_RESPONSE_JSON: &str = r#"{
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "issuance_factory_id": "660e8400-e29b-41d4-a716-446655440001",
        "status": "active",
        "collateral_asset": "030201",
        "principal_asset": "060504",
        "collateral_amount": "1000",
        "principal_amount": "500",
        "interest_rate": 120,
        "loan_expiration_height": 12345,
        "created_at_height": 100,
        "created_at_txid": "ccbbaa",
        "borrower_nft_asset": "0a09",
        "lender_nft_asset": "0c0b",
        "protocol_fee_keeper_asset": "2c0b",
        "participants": [{
            "offer_id": "550e8400-e29b-41d4-a716-446655440000",
            "participant_type": "borrower",
            "script_pubkey": "51ac",
            "txid": "030201",
            "vout": 0,
            "created_at_height": 100,
            "spent_txid": null,
            "spent_at_height": null
        }],
        "utxos": [{
            "offer_id": "550e8400-e29b-41d4-a716-446655440000",
            "txid": "aabb",
            "vout": 1,
            "utxo_type": "borrower_principal",
            "created_at_height": 100,
            "spent_txid": null,
            "spent_at_height": null
        }]
    }"#;

    const OVERVIEW_JSON: &str = r#"{
        "collateral_locked": [{"asset": "030201", "amount": "1000"}],
        "active_loan_principal": [{"asset": "060504", "amount": "500"}],
        "active_loans_count": 1
    }"#;

    #[test]
    fn deserializes_offer_list_response() {
        let response: LendingOfferListResponse =
            serde_json::from_str(LIST_RESPONSE_JSON).expect("parse list");
        assert_eq!(response.total, 1);
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].status, LendingOfferStatus::Pending);
        assert_eq!(response.items[0].participants.len(), 1);
    }

    #[test]
    fn deserializes_offer_details_response() {
        let details = parse_offer_details_json(DETAILS_RESPONSE_JSON).expect("parse details");
        assert_eq!(details.status, LendingOfferStatus::Active);
        assert_eq!(details.borrower_nft_asset, "0a09");
        assert_eq!(details.participants.len(), 1);
        assert_eq!(details.utxos[0].utxo_type, LendingUtxoType::BorrowerPrincipal);
    }

    #[test]
    fn deserializes_offers_overview() {
        let overview: LendingOffersOverview =
            serde_json::from_str(OVERVIEW_JSON).expect("parse overview");
        assert_eq!(overview.active_loans_count, 1);
        assert_eq!(overview.collateral_locked[0].amount, "1000");
    }

    #[test]
    fn deserializes_offer_ids_by_script() {
        let ids: Vec<String> =
            serde_json::from_str(r#"["550e8400-e29b-41d4-a716-446655440000"]"#).expect("parse ids");
        assert_eq!(ids.len(), 1);
    }

    #[test]
    fn list_query_builds_expected_pairs() {
        let query = LendingOfferListQuery {
            status: Some(vec![LendingOfferStatus::Pending, LendingOfferStatus::Active]),
            collateral_asset: Some("030201".to_string()),
            limit: Some(10),
            offset: Some(5),
            ..Default::default()
        };

        let pairs = query.to_query_pairs();
        assert!(pairs.contains(&("status".to_string(), "pending,active".to_string())));
        assert!(pairs.contains(&("collateral_asset".to_string(), "030201".to_string())));
        assert!(pairs.contains(&("limit".to_string(), "10".to_string())));
        assert!(pairs.contains(&("offset".to_string(), "5".to_string())));
        assert!(pairs.contains(&(
            "sort_by".to_string(),
            "created_at_height".to_string()
        )));
    }
}
