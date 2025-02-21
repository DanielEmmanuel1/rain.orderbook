use super::common::*;
use crate::schema;
use serde::Serialize;
#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::prelude::*;

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(
    graphql_type = "Query",
    variables = "SgPaginationWithTimestampQueryVariables"
)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrderTradesListQuery {
    #[arguments(
        skip: $skip,
        first: $first,
        orderBy: "timestamp",
        orderDirection: "desc",
        where: {
            order_: { id: $id },
            timestamp_gte: $timestamp_gte,
            timestamp_lte: $timestamp_lte
        }
    )]
    pub trades: Vec<SgTrade>,
}

#[derive(cynic::QueryFragment, Debug, Clone, Serialize)]
#[cynic(graphql_type = "Query", variables = "SgIdQueryVariables")]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct SgOrderTradeDetailQuery {
    #[arguments(id: $id)]
    #[cfg_attr(target_family = "wasm", tsify(optional))]
    pub trade: Option<SgTrade>,
}
