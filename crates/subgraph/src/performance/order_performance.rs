use super::apy::APYDetails;
use super::vol::VolumeDetails;
use super::{PerformanceError, YEAR18};
use crate::performance::apy::{get_vaults_apy, TokenPair};
use crate::{
    performance::vol::get_vaults_vol,
    types::common::{Erc20, Order, Trade},
};
use alloy::primitives::U256;
use rain_orderbook_math::{BigUintMath, ONE18};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use typeshare::typeshare;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct VaultPerformance {
    /// vault id
    pub id: String,
    /// vault token
    pub token: Erc20,
    /// vault vol segment
    pub vol_details: VolumeDetails,
    /// vault apy segment
    pub apy_details: Option<APYDetails>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct DenominatedPerformance {
    /// The denomination token
    pub token: Erc20,
    /// Order's APY raw value
    pub apy: U256,
    /// Determines if apy is negative or not
    pub apy_is_neg: bool,
    /// Order's net vol raw value
    pub net_vol: U256,
    /// Determines if net_vol is negative or not
    pub net_vol_is_neg: bool,
    /// Order's starting capital
    pub starting_capital: U256,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[typeshare]
pub struct OrderPerformance {
    /// Order subgraph id
    pub order_id: String,
    /// Order hash
    pub order_hash: String,
    /// Order's orderbook
    pub orderbook: String,
    /// Order's measured performance as a whole
    pub denominated_performance: Option<DenominatedPerformance>,
    /// Start timestamp of the performance measring timeframe
    #[typeshare(typescript(type = "number"))]
    pub start_time: u64,
    /// End timestamp of the performance measuring timeframe
    #[typeshare(typescript(type = "number"))]
    pub end_time: u64,
    /// Order's input vaults isolated performance
    pub inputs_vaults: Vec<VaultPerformance>,
    /// Order's output vaults isolated performance
    pub outputs_vaults: Vec<VaultPerformance>,
}

impl OrderPerformance {
    /// Given an order and its trades and optionally a timeframe, will calculates
    /// the order performance, (apy and volume)
    /// Trades must be sorted in desc order by timestamp, this is the case if
    /// queried from subgraph using this lib functionalities
    pub fn measure(
        order: &Order,
        trades: &[Trade],
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<OrderPerformance, PerformanceError> {
        // return early if there are no trades
        if trades.is_empty() {
            return Ok(OrderPerformance {
                order_id: order.id.0.clone(),
                order_hash: order.order_hash.0.clone(),
                orderbook: order.orderbook.id.0.clone(),
                start_time: start_timestamp.unwrap_or(0),
                end_time: end_timestamp.unwrap_or(chrono::Utc::now().timestamp() as u64),
                inputs_vaults: vec![],
                outputs_vaults: vec![],
                denominated_performance: None,
            });
        }
        let vaults_vol = get_vaults_vol(trades)?;
        let vaults_apy = get_vaults_apy(trades, &vaults_vol, start_timestamp, end_timestamp)?;

        // build an OrderPerformance struct
        // pick the order's whole performance timeframe from the vaults biggest timeframe
        // and put the calculated vaults vol and apy into inputs and outputs vaults fields
        let mut start_time = u64::MAX;
        let mut end_time = 0_u64;
        let mut inputs: Vec<VaultPerformance> = vec![];
        let mut outputs: Vec<VaultPerformance> = vec![];
        for (vault_apy, vault_vol) in vaults_apy.iter().zip(vaults_vol) {
            vault_apy.apy_details.inspect(|v| {
                if v.start_time < start_time {
                    start_time = v.start_time;
                }
                if v.end_time > end_time {
                    end_time = v.end_time;
                }
            });
            if order
                .inputs
                .iter()
                .any(|v| v.vault_id.0 == vault_apy.id && v.token == vault_apy.token)
            {
                inputs.push(VaultPerformance {
                    id: vault_apy.id.clone(),
                    token: vault_apy.token.clone(),
                    apy_details: vault_apy.apy_details,
                    vol_details: vault_vol.vol_details,
                });
            }
            if order
                .outputs
                .iter()
                .any(|v| v.vault_id.0 == vault_apy.id && v.token == vault_apy.token)
            {
                outputs.push(VaultPerformance {
                    id: vault_apy.id.clone(),
                    token: vault_apy.token.clone(),
                    apy_details: vault_apy.apy_details,
                    vol_details: vault_vol.vol_details,
                });
            }
        }
        let mut order_performance = OrderPerformance {
            order_id: order.id.0.clone(),
            order_hash: order.order_hash.0.clone(),
            orderbook: order.orderbook.id.0.clone(),
            start_time,
            end_time,
            inputs_vaults: inputs,
            outputs_vaults: outputs,
            denominated_performance: None,
        };

        // get pairs ratios
        let pair_ratio_map = get_order_pairs_ratio(order, trades);

        // try to calculate all vaults capital and volume denominated into each of
        // the order's tokens by checking if there is direct ratio between the tokens,
        // multi path ratios are ignored currently and results in None for the APY.
        // if there is a success for any of the denomination tokens, gather it in order
        // of its net vol and pick the one with highest net vol.
        // if there was no success with any of the order's tokens, simply return None
        // for the APY.
        let mut processed_tokens: Vec<&Erc20> = vec![];
        let mut all_tokens_vols_list: Vec<TokenDenominationVol> = vec![];
        let mut full_apy_in_distinct_token_denominations = vec![];
        for token in &vaults_apy {
            // skip if token is alreaedy processed
            if processed_tokens.contains(&&token.token) {
                continue;
            } else {
                processed_tokens.push(&token.token);
            }
            let mut noway = false;
            let mut net_vol_is_neg = false;
            let mut net_vol_rate_is_neg = false;
            let mut combined_capital = U256::ZERO;
            let mut combined_net_vol = U256::ZERO;
            let mut combined_annual_rate_vol = U256::ZERO;
            let mut current_token_vol_list: Vec<TokenDenominationVol> = vec![];
            for token_vault in &vaults_apy {
                if let Some(apy_details) = token_vault.apy_details {
                    // a closure fn handles net vol combination
                    let mut handle_combined_net_vol = |new_net_vol: U256| {
                        if apy_details.is_neg == net_vol_is_neg {
                            combined_net_vol += new_net_vol;
                        } else if net_vol_is_neg {
                            if new_net_vol >= combined_net_vol {
                                net_vol_is_neg = false;
                                combined_net_vol = new_net_vol - combined_net_vol;
                            } else {
                                combined_net_vol -= new_net_vol;
                            }
                        } else if combined_net_vol >= new_net_vol {
                            combined_net_vol -= new_net_vol;
                        } else {
                            net_vol_is_neg = true;
                            combined_net_vol = new_net_vol - combined_net_vol;
                        }
                    };

                    // a closure fn handles annual rate vol combination
                    let mut handle_combined_annual_rate_vol = |new_annual_rate_vol: U256| {
                        if apy_details.is_neg == net_vol_rate_is_neg {
                            combined_annual_rate_vol += new_annual_rate_vol;
                        } else if net_vol_rate_is_neg {
                            if new_annual_rate_vol >= combined_annual_rate_vol {
                                net_vol_rate_is_neg = false;
                                combined_annual_rate_vol =
                                    new_annual_rate_vol - combined_annual_rate_vol;
                            } else {
                                combined_annual_rate_vol -= new_annual_rate_vol;
                            }
                        } else if combined_annual_rate_vol >= new_annual_rate_vol {
                            combined_annual_rate_vol -= new_annual_rate_vol;
                        } else {
                            net_vol_rate_is_neg = true;
                            combined_annual_rate_vol =
                                new_annual_rate_vol - combined_annual_rate_vol;
                        }
                    };

                    // this vault's timeframe to year ratio
                    let annual_rate = U256::from(apy_details.end_time - apy_details.start_time)
                        .saturating_mul(ONE18)
                        .div_18(*YEAR18)
                        .map_err(PerformanceError::from)?;

                    // sum up all token vaults' capitals and vols in the current's iteration's
                    // token denomination by using the direct ratio between the tokens
                    if token_vault.token == token.token {
                        combined_capital += apy_details.capital;
                        handle_combined_net_vol(apy_details.net_vol);

                        let annual_rate_vol = apy_details
                            .net_vol
                            .div_18(annual_rate)
                            .map_err(PerformanceError::from)?;
                        handle_combined_annual_rate_vol(annual_rate_vol);

                        current_token_vol_list.push(TokenDenominationVol {
                            net_vol: apy_details.net_vol,
                            is_neg: apy_details.is_neg,
                            token: &token.token,
                        });
                    } else {
                        let pair = TokenPair {
                            input: token.token.clone(),
                            output: token_vault.token.clone(),
                        };
                        // convert to current denomination by the direct pair ratio if exists
                        if let Some(Some(ratio)) = pair_ratio_map.get(&pair) {
                            let capital_converted = apy_details
                                .capital
                                .mul_18(*ratio)
                                .map_err(PerformanceError::from)?;
                            combined_capital += capital_converted;

                            let net_vol_converted = apy_details
                                .net_vol
                                .mul_18(*ratio)
                                .map_err(PerformanceError::from)?;
                            handle_combined_net_vol(net_vol_converted);

                            let annual_rate_vol_converted = net_vol_converted
                                .div_18(annual_rate)
                                .map_err(PerformanceError::from)?;
                            handle_combined_annual_rate_vol(annual_rate_vol_converted);

                            current_token_vol_list.push(TokenDenominationVol {
                                net_vol: net_vol_converted,
                                is_neg: apy_details.is_neg,
                                token: &token_vault.token,
                            });
                        } else {
                            noway = true;
                            break;
                        }
                    }
                }
            }

            // for every success apy calc in a token denomination, gather them in an array,
            // this means at the end we have all the successful apy calculated in each of
            // the order's io tokens in order from highest to lowest when sorted.
            if !noway {
                if let Ok(apy) = combined_annual_rate_vol.div_18(combined_capital) {
                    full_apy_in_distinct_token_denominations.push(DenominatedPerformance {
                        apy,
                        apy_is_neg: net_vol_rate_is_neg,
                        token: token.token.clone(),
                        starting_capital: combined_capital,
                        net_vol: combined_net_vol,
                        net_vol_is_neg,
                    });
                }
            } else {
                current_token_vol_list.clear();
            }

            // if we already have ordered token net vol in a denomination
            // we dont need them in other denominations in order to pick
            // the highest vol token as settlement denomination
            if all_tokens_vols_list.is_empty() {
                all_tokens_vols_list.extend(current_token_vol_list);
            }
        }

        // after array is sorted, pick the denomination with highest net vol by
        // iterating over tokens with highest vol to lowest and pick the first matching one
        all_tokens_vols_list.sort();
        for token in all_tokens_vols_list.iter().rev() {
            if let Some(denominated_apy) = full_apy_in_distinct_token_denominations
                .iter()
                .find(|&v| &v.token == token.token)
            {
                order_performance.denominated_performance = Some(denominated_apy.clone());
                // return early as soon as a match is found
                return Ok(order_performance);
            }
        }

        Ok(order_performance)
    }
}

/// Calculates an order's pairs' ratios from their last trades in a given list of trades
/// Trades must be sorted in desc order by timestamp, this is the case if queried from subgraph
/// using this lib functionalities
pub fn get_order_pairs_ratio(order: &Order, trades: &[Trade]) -> HashMap<TokenPair, Option<U256>> {
    let mut pair_ratio_map: HashMap<TokenPair, Option<U256>> = HashMap::new();
    for input in &order.inputs {
        for output in &order.outputs {
            let pair_as_key = TokenPair {
                input: input.token.clone(),
                output: output.token.clone(),
            };
            let inverse_pair_as_key = TokenPair {
                input: output.token.clone(),
                output: input.token.clone(),
            };
            // if not same io token and ratio map doesnt already include them
            if input.token != output.token
                && !(pair_ratio_map.contains_key(&pair_as_key)
                    || pair_ratio_map.contains_key(&inverse_pair_as_key))
            {
                // find this pairs(io or oi) latest tradetrades from list of order's
                // trades, the calculate the pair ratio (in amount/out amount) and
                // its inverse from the latest trade that involes these 2 tokens.
                let ratio = trades
                    .iter()
                    .find(|v| {
                        (v.input_vault_balance_change.vault.token == input.token
                            && v.output_vault_balance_change.vault.token == output.token)
                            || (v.output_vault_balance_change.vault.token == input.token
                                && v.input_vault_balance_change.vault.token == output.token)
                    })
                    .and_then(|latest_trade| {
                        // convert input and output amounts to 18 decimals point
                        // and then calculate the pair ratio
                        latest_trade
                            .ratio()
                            .ok()
                            .zip(latest_trade.inverse_ratio().ok())
                            .map(|(ratio, inverse_ratio)| [ratio, inverse_ratio])
                    });

                // io
                pair_ratio_map.insert(pair_as_key, ratio.map(|v| v[0]));
                // oi
                pair_ratio_map.insert(inverse_pair_as_key, ratio.map(|v| v[1]));
            }
        }
    }

    pair_ratio_map
}

/// helper struct that provides sorting tokens based on their combined net vol
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TokenDenominationVol<'a> {
    token: &'a Erc20,
    net_vol: U256,
    is_neg: bool,
}
impl<'a> Ord for TokenDenominationVol<'a> {
    fn clamp(self, _min: Self, _max: Self) -> Self
    where
        Self: Sized,
        Self: PartialOrd,
    {
        self
    }
    fn cmp(&self, other: &Self) -> Ordering {
        if self.is_neg == other.is_neg {
            match self.net_vol.cmp(&other.net_vol) {
                Ordering::Greater => {
                    if self.is_neg {
                        Ordering::Less
                    } else {
                        Ordering::Greater
                    }
                }
                Ordering::Less => {
                    if self.is_neg {
                        Ordering::Greater
                    } else {
                        Ordering::Less
                    }
                }
                Ordering::Equal => Ordering::Equal,
            }
        } else if self.is_neg {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match self.cmp(&other) {
            Ordering::Greater => other,
            Ordering::Less => self,
            Ordering::Equal => self,
        }
    }
    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        match self.cmp(&other) {
            Ordering::Greater => self,
            Ordering::Less => other,
            Ordering::Equal => self,
        }
    }
}
impl<'a> PartialOrd for TokenDenominationVol<'a> {
    fn ge(&self, other: &Self) -> bool {
        !matches!(self.cmp(other), Ordering::Less)
    }
    fn le(&self, other: &Self) -> bool {
        !matches!(self.cmp(other), Ordering::Greater)
    }
    fn gt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }
    fn lt(&self, other: &Self) -> bool {
        matches!(self.cmp(other), Ordering::Less)
    }
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::common::{
        BigInt, Bytes, Order, Orderbook, TradeEvent, TradeStructPartialOrder,
        TradeVaultBalanceChange, Transaction, Vault, VaultBalanceChangeVault,
    };
    use alloy::primitives::{Address, B256};
    use std::str::FromStr;

    #[test]
    fn test_get_pairs_ratio() {
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let result = get_order_pairs_ratio(&get_order(), &trades);
        let mut expected = HashMap::new();
        expected.insert(
            TokenPair {
                input: token2.clone(),
                output: token1.clone(),
            },
            Some(U256::from_str("285714285714285714").unwrap()),
        );
        expected.insert(
            TokenPair {
                input: token1.clone(),
                output: token2.clone(),
            },
            Some(U256::from_str("3500000000000000000").unwrap()),
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_order_performance() {
        let order = get_order();
        let trades = get_trades();
        let [token1, token2] = get_tokens();
        let [vault1, vault2] = get_vault_ids();
        let token1_perf = VaultPerformance {
            id: vault1.to_string(),
            token: token1.clone(),
            apy_details: Some(APYDetails {
                start_time: 1,
                end_time: 10000001,
                net_vol: U256::from_str("5000000000000000000").unwrap(),
                capital: U256::from_str("5000000000000000000").unwrap(),
                apy: U256::from_str("3153600000000000000").unwrap(),
                is_neg: false,
            }),
            vol_details: VolumeDetails {
                net_vol: U256::from_str("5000000000000000000").unwrap(),
                total_in: U256::from_str("7000000000000000000").unwrap(),
                total_out: U256::from_str("2000000000000000000").unwrap(),
                total_vol: U256::from_str("9000000000000000000").unwrap(),
            },
        };
        let token2_perf = VaultPerformance {
            id: vault2.to_string(),
            token: token2.clone(),
            apy_details: Some(APYDetails {
                start_time: 1,
                end_time: 10000001,
                net_vol: U256::from_str("3000000000000000000").unwrap(),
                capital: U256::from_str("5000000000000000000").unwrap(),
                apy: U256::from_str("1892160000000000000").unwrap(),
                is_neg: false,
            }),
            vol_details: VolumeDetails {
                net_vol: U256::from_str("3000000000000000000").unwrap(),
                total_in: U256::from_str("5000000000000000000").unwrap(),
                total_out: U256::from_str("2000000000000000000").unwrap(),
                total_vol: U256::from_str("7000000000000000000").unwrap(),
            },
        };
        let result = OrderPerformance::measure(&order, &trades, Some(1), Some(10000001)).unwrap();
        let expected = OrderPerformance {
            order_id: "order-id".to_string(),
            order_hash: "".to_string(),
            orderbook: "".to_string(),
            start_time: 1,
            end_time: 10000001,
            inputs_vaults: vec![token1_perf.clone(), token2_perf.clone()],
            outputs_vaults: vec![token1_perf.clone(), token2_perf.clone()],
            denominated_performance: Some(DenominatedPerformance {
                apy: U256::from_str("2172479999999999999").unwrap(),
                apy_is_neg: false,
                token: token2,
                net_vol: U256::from_str("4428571428571428570").unwrap(),
                starting_capital: U256::from_str("6428571428571428570").unwrap(),
                net_vol_is_neg: false,
            }),
        };

        assert_eq!(result, expected);
    }

    fn get_vault_ids() -> [B256; 2] {
        [
            B256::from_slice(&[0x11u8; 32]),
            B256::from_slice(&[0x22u8; 32]),
        ]
    }
    fn get_tokens() -> [Erc20; 2] {
        let token1_address = Address::from_slice(&[0x11u8; 20]);
        let token2_address = Address::from_slice(&[0x22u8; 20]);
        let token1 = Erc20 {
            id: Bytes(token1_address.to_string()),
            address: Bytes(token1_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(BigInt(18.to_string())),
        };
        let token2 = Erc20 {
            id: Bytes(token2_address.to_string()),
            address: Bytes(token2_address.to_string()),
            name: Some("Token2".to_string()),
            symbol: Some("Token2".to_string()),
            decimals: Some(BigInt(18.to_string())),
        };
        [token1, token2]
    }
    fn get_order() -> Order {
        let [vault_id1, vault_id2] = get_vault_ids();
        let [token1, token2] = get_tokens();
        let vault1 = Vault {
            id: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            vault_id: BigInt(vault_id1.to_string()),
            balance: BigInt("".to_string()),
            token: token1,
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };
        let vault2 = Vault {
            id: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            vault_id: BigInt(vault_id2.to_string()),
            balance: BigInt("".to_string()),
            token: token2,
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            orders_as_output: vec![],
            orders_as_input: vec![],
            balance_changes: vec![],
        };
        Order {
            id: Bytes("order-id".to_string()),
            order_bytes: Bytes("".to_string()),
            order_hash: Bytes("".to_string()),
            owner: Bytes("".to_string()),
            outputs: vec![vault1.clone(), vault2.clone()],
            inputs: vec![vault1, vault2],
            orderbook: Orderbook {
                id: Bytes("".to_string()),
            },
            active: true,
            timestamp_added: BigInt("".to_string()),
            meta: None,
            add_events: vec![],
            trades: vec![],
        }
    }

    fn get_trades() -> Vec<Trade> {
        let bytes = Bytes("".to_string());
        let bigint = BigInt("".to_string());
        let [vault_id1, vault_id2] = get_vault_ids();
        let [token1, token2] = get_tokens();
        let trade1 = Trade {
            id: bytes.clone(),
            order: TradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: TradeEvent {
                sender: bytes.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: BigInt("1".to_string()),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2000000000000000000".to_string()),
                new_vault_balance: BigInt("2000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: BigInt("1".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("5000000000000000000".to_string()),
                new_vault_balance: BigInt("2000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: BigInt("1".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
        };
        let trade2 = Trade {
            id: bytes.clone(),
            order: TradeStructPartialOrder {
                id: bytes.clone(),
                order_hash: bytes.clone(),
            },
            trade_event: TradeEvent {
                sender: bytes.clone(),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: bigint.clone(),
                },
            },
            timestamp: BigInt("2".to_string()),
            orderbook: Orderbook { id: bytes.clone() },
            output_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("-2000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token2.clone(),
                    vault_id: BigInt(vault_id2.to_string()),
                },
                timestamp: BigInt("2".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
            input_vault_balance_change: TradeVaultBalanceChange {
                id: bytes.clone(),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: BigInt("7000000000000000000".to_string()),
                new_vault_balance: BigInt("5000000000000000000".to_string()),
                old_vault_balance: bigint.clone(),
                vault: VaultBalanceChangeVault {
                    id: bytes.clone(),
                    token: token1.clone(),
                    vault_id: BigInt(vault_id1.to_string()),
                },
                timestamp: BigInt("2".to_string()),
                transaction: Transaction {
                    id: bytes.clone(),
                    from: bytes.clone(),
                    block_number: bigint.clone(),
                    timestamp: BigInt("1".to_string()),
                },
                orderbook: Orderbook { id: bytes.clone() },
            },
        };
        vec![trade2, trade1]
    }
}
