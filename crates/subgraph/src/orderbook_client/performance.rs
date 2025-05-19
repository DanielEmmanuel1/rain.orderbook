use super::*;

impl OrderbookSubgraphClient {
    /// Fetch all pages of order_takes_list query and calculate vaults' vol
    pub async fn order_vaults_volume(
        &self,
        order_id: cynic::Id,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<Vec<VaultVolume>, OrderbookSubgraphClientError> {
        let trades = self
            .order_trades_list_all(order_id, start_timestamp, end_timestamp)
            .await?;
        Ok(get_vaults_vol(&trades)?)
    }

    /// Fetches order data and measures an order's detailed performance (apy and vol)
    pub async fn order_performance(
        &self,
        order_id: cynic::Id,
        start_timestamp: Option<u64>,
        end_timestamp: Option<u64>,
    ) -> Result<OrderPerformance, OrderbookSubgraphClientError> {
        let order = self.order_detail(order_id.clone()).await?;
        let trades = self
            .order_trades_list_all(order_id, start_timestamp, end_timestamp)
            .await?;
        Ok(OrderPerformance::measure(
            &order,
            &trades,
            start_timestamp,
            end_timestamp,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orderbook_client::OrderbookSubgraphClientError;
    use crate::performance::PerformanceError;
    use crate::types::common::{
        SgBigInt, SgBytes, SgErc20, SgOrder, SgOrderbook, SgTrade, SgTradeEvent,
        SgTradeStructPartialOrder, SgTradeVaultBalanceChange, SgTransaction, SgVault,
        SgVaultBalanceChangeVault,
    };
    use alloy::primitives::U256;
    use cynic::Id;
    use httpmock::prelude::*;
    use reqwest::Url;
    use serde_json::json;

    const ALL_PAGES_QUERY_PAGE_SIZE: i32 = 200;

    fn setup_client(server: &MockServer) -> OrderbookSubgraphClient {
        let url = Url::parse(&server.url("")).unwrap();
        OrderbookSubgraphClient::new(url)
    }

    fn default_sg_erc20(id_suffix: &str) -> SgErc20 {
        SgErc20 {
            id: SgBytes(format!("0xtoken_id_{}", id_suffix)),
            address: SgBytes(format!("0xtoken_address_{}", id_suffix)),
            name: Some(format!("Token {}", id_suffix.to_uppercase())),
            symbol: Some(format!("TKN{}", id_suffix.to_uppercase())),
            decimals: Some(SgBigInt("18".to_string())),
        }
    }

    fn default_sg_order(order_id_str: &str) -> SgOrder {
        SgOrder {
            id: SgBytes(order_id_str.to_string()),
            order_hash: SgBytes(format!("0xhash_{}", order_id_str)),
            owner: SgBytes("0xowner_default".to_string()),
            order_bytes: SgBytes("0xorderbytes_default".to_string()),
            timestamp_added: SgBigInt("1600000000".to_string()),
            active: true,
            orderbook: SgOrderbook {
                id: SgBytes("0xorderbook_default".to_string()),
            },
            inputs: vec![SgVault {
                id: SgBytes("input_vault_id".to_string()),
                owner: SgBytes("0xowner_default".to_string()),
                vault_id: SgBigInt("input_vault_sg_id".to_string()),
                balance: SgBigInt("1000".to_string()),
                token: default_sg_erc20("input"),
                orderbook: SgOrderbook {
                    id: SgBytes("0xorderbook_default".to_string()),
                },
                orders_as_output: vec![],
                orders_as_input: vec![],
                balance_changes: vec![],
            }],
            outputs: vec![SgVault {
                id: SgBytes("output_vault_id".to_string()),
                owner: SgBytes("0xowner_default".to_string()),
                vault_id: SgBigInt("output_vault_sg_id".to_string()),
                balance: SgBigInt("0".to_string()),
                token: default_sg_erc20("output"),
                orderbook: SgOrderbook {
                    id: SgBytes("0xorderbook_default".to_string()),
                },
                orders_as_output: vec![],
                orders_as_input: vec![],
                balance_changes: vec![],
            }],
            meta: None,
            add_events: vec![],
            trades: vec![],
            remove_events: vec![],
        }
    }

    fn default_sg_trade(trade_id_str: &str, order_id_str: &str, timestamp: u64) -> SgTrade {
        let input_token = default_sg_erc20("input");
        let output_token = default_sg_erc20("output");
        SgTrade {
            id: SgBytes(trade_id_str.to_string()),
            timestamp: SgBigInt(timestamp.to_string()),
            order: SgTradeStructPartialOrder {
                id: SgBytes(order_id_str.to_string()),
                order_hash: SgBytes(format!("0xhash_{}", order_id_str)),
            },
            orderbook: SgOrderbook {
                id: SgBytes("0xorderbook_default".to_string()),
            },
            trade_event: SgTradeEvent {
                transaction: SgTransaction {
                    id: SgBytes("0xtx_default".to_string()),
                    from: SgBytes("0xfrom_default".to_string()),
                    block_number: SgBigInt("1".to_string()),
                    timestamp: SgBigInt(timestamp.to_string()),
                },
                sender: SgBytes("0xsender_default".to_string()),
            },
            input_vault_balance_change: SgTradeVaultBalanceChange {
                id: SgBytes("ivbc_default".to_string()),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("100".to_string()),
                new_vault_balance: SgBigInt("1100".to_string()),
                old_vault_balance: SgBigInt("1000".to_string()),
                vault: SgVaultBalanceChangeVault {
                    id: SgBytes("input_vault_id".to_string()),
                    vault_id: SgBigInt("input_vault_sg_id".to_string()),
                    token: input_token.clone(),
                },
                timestamp: SgBigInt(timestamp.to_string()),
                transaction: SgTransaction {
                    id: SgBytes("0xtx_default".to_string()),
                    from: SgBytes("0xfrom_default".to_string()),
                    block_number: SgBigInt("1".to_string()),
                    timestamp: SgBigInt(timestamp.to_string()),
                },
                orderbook: SgOrderbook {
                    id: SgBytes("0xorderbook_default".to_string()),
                },
            },
            output_vault_balance_change: SgTradeVaultBalanceChange {
                id: SgBytes("ovbc_default".to_string()),
                __typename: "TradeVaultBalanceChange".to_string(),
                amount: SgBigInt("-50".to_string()),
                new_vault_balance: SgBigInt("50".to_string()),
                old_vault_balance: SgBigInt("100".to_string()),
                vault: SgVaultBalanceChangeVault {
                    id: SgBytes("output_vault_id".to_string()),
                    vault_id: SgBigInt("output_vault_sg_id".to_string()),
                    token: output_token.clone(),
                },
                timestamp: SgBigInt(timestamp.to_string()),
                transaction: SgTransaction {
                    id: SgBytes("0xtx_default".to_string()),
                    from: SgBytes("0xfrom_default".to_string()),
                    block_number: SgBigInt("1".to_string()),
                    timestamp: SgBigInt(timestamp.to_string()),
                },
                orderbook: SgOrderbook {
                    id: SgBytes("0xorderbook_default".to_string()),
                },
            },
        }
    }

    #[tokio::test]
    async fn test_order_vaults_volume_success() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0xvol_order_1";
        let order_id = Id::new(order_id_str);
        let trades = vec![default_sg_trade("trade1", order_id_str, 1600000100)];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"skip\":{}", 0));
            then.status(200)
                .json_body(json!({"data": {"trades": trades}}));
        });
        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE));
            then.status(200).json_body(json!({"data": {"trades": []}}));
        });

        let result = client.order_vaults_volume(order_id, None, None).await;
        assert!(result.is_ok(), "Result was: {:?}", result);
        let vault_volumes = result.unwrap();
        assert_eq!(vault_volumes[0].id, "input_vault_sg_id");
        assert_eq!(vault_volumes[0].token, default_sg_erc20("input"));
        assert_eq!(vault_volumes[0].vol_details.net_vol, U256::from(100));
        assert_eq!(vault_volumes[0].vol_details.total_in, U256::from(100));
        assert_eq!(vault_volumes[0].vol_details.total_out, U256::from(0));
        assert_eq!(vault_volumes[0].vol_details.total_vol, U256::from(100));
        assert_eq!(vault_volumes[1].id, "output_vault_sg_id");
        assert_eq!(vault_volumes[1].token, default_sg_erc20("output"));
        assert_eq!(vault_volumes[1].vol_details.net_vol, U256::from(50));
        assert_eq!(vault_volumes[1].vol_details.total_in, U256::from(0));
        assert_eq!(vault_volumes[1].vol_details.total_out, U256::from(50));
        assert_eq!(vault_volumes[1].vol_details.total_vol, U256::from(50));
    }

    #[tokio::test]
    async fn test_order_vaults_volume_success_no_trades() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0xvol_order_notrades";
        let order_id = Id::new(order_id_str);

        sg_server.mock(|when, then| {
            when.method(POST).path("/").body_contains(order_id_str);
            then.status(200).json_body(json!({"data": {"trades": []}}));
        });

        let result = client.order_vaults_volume(order_id, None, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_order_vaults_volume_error_from_trades_list_all() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id = Id::new("0xvol_order_err_trades");

        sg_server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(500);
        });

        let result = client.order_vaults_volume(order_id, None, None).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }

    #[tokio::test]
    async fn test_order_performance_success() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0xperf_order_1";
        let order_id = Id::new(order_id_str);
        let order_data = default_sg_order(order_id_str);
        let trades_data = vec![
            default_sg_trade("trade_perf_1", order_id_str, 1600000100),
            default_sg_trade("trade_perf_2", order_id_str, 1600000200),
        ];

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", order_id_str))
                .body_contains("SgOrderDetailByIdQuery");
            then.status(200)
                .json_body(json!({"data": {"order": order_data}}));
        });

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(order_id_str)
                .body_contains("SgOrderTradesListQuery")
                .body_contains("\"skip\":0");
            then.status(200)
                .json_body(json!({"data": {"trades": trades_data}}));
        });

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(order_id_str)
                .body_contains("SgOrderTradesListQuery")
                .body_contains(format!("\"skip\":{}", ALL_PAGES_QUERY_PAGE_SIZE));
            then.status(200).json_body(json!({"data": {"trades": []}}));
        });

        let result = client.order_performance(order_id, None, None).await;
        assert!(result.is_ok(), "Result was: {:?}", result);
        let performance_report = result.unwrap();
        assert_eq!(performance_report.order_id, order_id_str);
        assert_eq!(performance_report.inputs_vaults.len(), 1);
        assert_eq!(performance_report.outputs_vaults.len(), 1);
        assert_eq!(performance_report.inputs_vaults[0].id, "input_vault_sg_id");
        assert_eq!(
            performance_report.inputs_vaults[0].token,
            default_sg_erc20("input")
        );
        assert_eq!(
            performance_report.outputs_vaults[0].id,
            "output_vault_sg_id"
        );
        assert_eq!(
            performance_report.outputs_vaults[0].token,
            default_sg_erc20("output")
        );
        assert_eq!(
            performance_report.inputs_vaults[0].vol_details.net_vol,
            U256::from(200)
        );
        assert_eq!(
            performance_report.inputs_vaults[0].vol_details.total_in,
            U256::from(200)
        );
        assert_eq!(
            performance_report.inputs_vaults[0].vol_details.total_out,
            U256::from(0)
        );
        assert_eq!(
            performance_report.inputs_vaults[0].vol_details.total_vol,
            U256::from(200)
        );
        assert_eq!(
            performance_report.outputs_vaults[0].vol_details.net_vol,
            U256::from(100)
        );
        assert_eq!(
            performance_report.outputs_vaults[0].vol_details.total_in,
            U256::from(0)
        );
        assert_eq!(
            performance_report.outputs_vaults[0].vol_details.total_out,
            U256::from(100)
        );
        assert_eq!(
            performance_report.outputs_vaults[0].vol_details.total_vol,
            U256::from(100)
        );
    }

    #[tokio::test]
    async fn test_order_performance_error_no_trades() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0xperf_order_notrades";
        let order_id = Id::new(order_id_str);
        let order_data = default_sg_order(order_id_str);

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", order_id_str))
                .body_contains("SgOrderDetailByIdQuery");
            then.status(200)
                .json_body(json!({"data": {"order": order_data}}));
        });

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(order_id_str)
                .body_contains("SgOrderTradesListQuery");
            then.status(200).json_body(json!({"data": {"trades": []}}));
        });

        let result = client.order_performance(order_id, None, None).await;
        assert!(result.is_err());
        match result {
            Err(OrderbookSubgraphClientError::PerformanceError(PerformanceError::NoTrades)) => (),
            _ => panic!("Expected PerformanceError::NoTrades, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_order_performance_error_from_order_detail() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id = Id::new("0xperf_order_err_detail");

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains("SgOrderDetailByIdQuery");
            then.status(200).json_body(json!({"data": {"order": null}}));
        });

        let result = client.order_performance(order_id, None, None).await;
        assert!(matches!(result, Err(OrderbookSubgraphClientError::Empty)));
    }

    #[tokio::test]
    async fn test_order_performance_error_from_trades_list_all() {
        let sg_server = MockServer::start_async().await;
        let client = setup_client(&sg_server);
        let order_id_str = "0xperf_order_err_trades";
        let order_id = Id::new(order_id_str);
        let order_data = default_sg_order(order_id_str);

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(format!("\"id\":\"{}\"", order_id_str))
                .body_contains("SgOrderDetailByIdQuery");
            then.status(200)
                .json_body(json!({"data": {"order": order_data}}));
        });

        sg_server.mock(|when, then| {
            when.method(POST)
                .path("/")
                .body_contains(order_id_str)
                .body_contains("SgOrderTradesListQuery");
            then.status(500);
        });

        let result = client.order_performance(order_id, None, None).await;
        assert!(matches!(
            result,
            Err(OrderbookSubgraphClientError::CynicClientError(_))
        ));
    }
}
