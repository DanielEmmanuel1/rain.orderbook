#[cfg(not(target_family = "wasm"))]
pub use rain_interpreter_eval::trace::*;
use rain_orderbook_app_settings::chart::ChartCfg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(target_family = "wasm")]
use wasm_bindgen_utils::{impl_wasm_traits, prelude::*, serialize_hashmap_as_object};

#[cfg(not(target_family = "wasm"))]
mod impls;
#[cfg(not(target_family = "wasm"))]
pub use impls::*;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ChartData {
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, FuzzResultFlat>")
    )]
    pub scenarios_data: HashMap<String, FuzzResultFlat>,
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, ChartCfg>")
    )]
    pub charts: HashMap<String, ChartCfg>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(ChartData);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FuzzResultFlat {
    pub scenario: String,
    pub data: RainEvalResultsTable,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(FuzzResultFlat);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DeploymentsDebugDataMap {
    #[cfg_attr(
        target_family = "wasm",
        serde(serialize_with = "serialize_hashmap_as_object"),
        tsify(type = "Record<string, DeploymentDebugData>")
    )]
    pub data_map: HashMap<String, DeploymentDebugData>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentsDebugDataMap);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct DeploymentDebugData {
    pub pairs_data: Vec<DeploymentDebugPairData>,
    pub block_number: u64,
    pub chain_id: u64,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentDebugData);

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct DeploymentDebugPairData {
    pub order: String,
    pub scenario: String,
    pub pair: String,
    pub result: Option<FuzzResultFlat>,
    pub error: Option<String>,
}
#[cfg(target_family = "wasm")]
impl_wasm_traits!(DeploymentDebugPairData);

// Stub definitions for wasm builds where the full tracing machinery is unavailable.
// These provide the minimum surface required by the TypeScript bindings (mainly the
// ability to serialise/deserialize and be exposed through tsify).
#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RainEvalResultsTable;

#[cfg(target_family = "wasm")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[cfg_attr(target_family = "wasm", derive(Tsify))]
pub struct RainEvalResults;

#[cfg(target_family = "wasm")]
impl RainEvalResults {
    #[allow(clippy::unused_self)]
    pub fn into_flattened_table(self) -> RainEvalResultsTable {
        RainEvalResultsTable
    }
}
