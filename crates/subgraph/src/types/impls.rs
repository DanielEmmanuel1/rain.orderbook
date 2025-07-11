use super::common::*;
use crate::performance::PerformanceError;
use rain_math_float::Float;

impl SgErc20 {
    pub fn get_decimals(&self) -> Result<u8, PerformanceError> {
        Ok(self
            .decimals
            .as_ref()
            .map(|v| v.0.as_str())
            .unwrap_or("18")
            .parse()?)
    }
}

impl SgTrade {
    /// Calculates the trade's I/O ratio
    pub fn ratio(&self) -> Result<Float, PerformanceError> {
        let input = Float::parse(self.input_vault_balance_change.amount.0.clone())?;
        let output = Float::parse(self.output_vault_balance_change.amount.0.clone())?;

        if output.is_zero()? {
            Err(PerformanceError::DivByZero)
        } else {
            Ok((input / output)?)
        }
    }

    /// Calculates the trade's O/I ratio (inverse)
    pub fn inverse_ratio(&self) -> Result<Float, PerformanceError> {
        let input = Float::parse(self.input_vault_balance_change.amount.0.clone())?;
        let output = Float::parse(self.output_vault_balance_change.amount.0.clone())?;

        if output.is_zero()? {
            Err(PerformanceError::DivByZero)
        } else {
            Ok((output / input)?)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::common::{
        SgBigInt, SgBytes, SgOrderbook, SgTradeEvent, SgTradeStructPartialOrder,
        SgTradeVaultBalanceChange, SgTransaction, SgVaultBalanceChangeVault,
    };

    use alloy::primitives::{
        ruint::{BaseConvertError, ParseError},
        Address,
    };

    #[test]
    fn test_token_get_decimals_ok() {
        // known decimals
        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(6.to_string())),
        };
        let result = token.get_decimals().unwrap();
        assert_eq!(result, 6);

        // unknown decimals, defaults to 18
        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: None,
        };
        let result = token.get_decimals().unwrap();
        assert_eq!(result, 18);
    }

    #[test]
    fn test_token_get_decimals_err() {
        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt("".to_string())),
        };
        let err = token.get_decimals().unwrap_err();
        assert!(matches!(err, PerformanceError::ParseIntError(_)));

        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt("not a number".to_string())),
        };
        let err = token.get_decimals().unwrap_err();
        assert!(matches!(err, PerformanceError::ParseIntError(_)));

        let token = SgErc20 {
            id: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            address: SgBytes(Address::from_slice(&[0x11u8; 20]).to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt("-1".to_string())),
        };
        let err = token.get_decimals().unwrap_err();
        assert!(matches!(err, PerformanceError::ParseIntError(_)));
    }

    #[test]
    fn test_ratio_happy() {
        let result = get_trade().ratio().unwrap();
        let expected = Float::parse("500000000000000000".to_string()).unwrap();
        assert!(result.eq(expected).unwrap());
    }

    #[test]
    fn test_ratio_unhappy() {
        let mut trade = get_trade();
        let amount = Float::parse("0".to_string()).unwrap();
        let amount_str = serde_json::to_string(&amount).unwrap();
        trade.output_vault_balance_change.amount = SgBytes(amount_str);
        matches!(trade.ratio().unwrap_err(), PerformanceError::DivByZero);
    }

    #[test]
    fn test_inverse_ratio_happy() {
        let result = get_trade().inverse_ratio().unwrap();
        let expected = Float::parse("2000000000000000000".to_string()).unwrap();
        assert!(result.eq(expected).unwrap());
    }

    #[test]
    fn test_inverse_ratio_unhappy() {
        let mut trade = get_trade();
        let amount = Float::parse("0".to_string()).unwrap();
        let amount_str = serde_json::to_string(&amount).unwrap();
        trade.input_vault_balance_change.amount = SgBytes(amount_str);
        matches!(
            trade.inverse_ratio().unwrap_err(),
            PerformanceError::DivByZero
        );
    }

    // helper to get trade struct
    fn get_trade() -> SgTrade {
        let token_address = Address::from_slice(&[0x11u8; 20]);
        let token = SgErc20 {
            id: SgBytes(token_address.to_string()),
            address: SgBytes(token_address.to_string()),
            name: Some("Token1".to_string()),
            symbol: Some("Token1".to_string()),
            decimals: Some(SgBigInt(6.to_string())),
        };

        let amount = Float::parse("3000000".to_string()).unwrap();
        let amount_str = serde_json::to_string(&amount).unwrap();

        let input_trade_vault_balance_change = SgTradeVaultBalanceChange {
            id: SgBytes("".to_string()),
            __typename: "".to_string(),
            amount: SgBytes(amount_str),
            new_vault_balance: SgBytes("".to_string()),
            old_vault_balance: SgBytes("".to_string()),
            vault: SgVaultBalanceChangeVault {
                id: SgBytes("".to_string()),
                vault_id: SgBytes("".to_string()),
                token: token.clone(),
            },
            timestamp: SgBigInt("".to_string()),
            transaction: SgTransaction {
                id: SgBytes("".to_string()),
                from: SgBytes("".to_string()),
                block_number: SgBigInt("".to_string()),
                timestamp: SgBigInt("".to_string()),
            },
            orderbook: SgOrderbook {
                id: SgBytes("".to_string()),
            },
        };

        let amount = Float::parse("-6000000".to_string()).unwrap();
        let amount_str = serde_json::to_string(&amount).unwrap();

        let output_trade_vault_balance_change = SgTradeVaultBalanceChange {
            id: SgBytes("".to_string()),
            __typename: "".to_string(),
            amount: SgBytes(amount_str),
            new_vault_balance: SgBytes("".to_string()),
            old_vault_balance: SgBytes("".to_string()),
            vault: SgVaultBalanceChangeVault {
                id: SgBytes("".to_string()),
                vault_id: SgBytes("".to_string()),
                token: token.clone(),
            },
            timestamp: SgBigInt("".to_string()),
            transaction: SgTransaction {
                id: SgBytes("".to_string()),
                from: SgBytes("".to_string()),
                block_number: SgBigInt("".to_string()),
                timestamp: SgBigInt("".to_string()),
            },
            orderbook: SgOrderbook {
                id: SgBytes("".to_string()),
            },
        };

        SgTrade {
            id: SgBytes("".to_string()),
            trade_event: SgTradeEvent {
                transaction: SgTransaction {
                    id: SgBytes("".to_string()),
                    from: SgBytes("".to_string()),
                    block_number: SgBigInt("".to_string()),
                    timestamp: SgBigInt("".to_string()),
                },
                sender: SgBytes("".to_string()),
            },
            output_vault_balance_change: output_trade_vault_balance_change,
            input_vault_balance_change: input_trade_vault_balance_change,
            order: SgTradeStructPartialOrder {
                id: SgBytes("".to_string()),
                order_hash: SgBytes("".to_string()),
            },
            timestamp: SgBigInt("".to_string()),
            orderbook: SgOrderbook {
                id: SgBytes("".to_string()),
            },
        }
    }
}
