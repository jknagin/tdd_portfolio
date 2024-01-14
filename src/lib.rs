use chrono::NaiveDateTime;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
enum TransactionType {
    Purchase,
    Sell,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
struct PurchaseRecord {
    date: NaiveDateTime,
    shares: u32,
    transaction_type: TransactionType,
}

struct Portfolio {
    holdings: HashMap<String, u32>,
    purchase_records: HashMap<String, Vec<PurchaseRecord>>,
}

#[derive(Debug, thiserror::Error)]
enum PortfolioError {
    #[error("Cannot perform transaction with zero shares")]
    ZeroShares,

    #[error("Cannot sell more shares than owned")]
    InvalidSell,

    #[error("No history for symbol")]
    NoSymbolHistory,

    #[error("Too many shares puchased")]
    InvalidPurchase,
}

type PortfolioResult<T> = Result<T, PortfolioError>;

#[allow(dead_code)]
impl Portfolio {
    const FIXED_EPOCH_TIME_MS: i64 = 0;

    const EMPTY_PURCHASE_RECORD: Vec<PurchaseRecord> = vec![];

    fn fixed_date_time() -> NaiveDateTime {
        NaiveDateTime::from_timestamp_millis(Self::FIXED_EPOCH_TIME_MS).unwrap()
    }

    fn new() -> Self {
        Self {
            holdings: HashMap::new(),
            purchase_records: HashMap::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.holdings.is_empty()
    }

    fn validate_share_count(shares: u32) -> PortfolioResult<()> {
        if shares == 0 {
            return Err(PortfolioError::ZeroShares);
        }
        Ok(())
    }

    fn purchase(&mut self, symbol: &str, shares: u32) -> PortfolioResult<()> {
        self.transact(symbol, shares, TransactionType::Purchase)
    }

    fn sell(&mut self, symbol: &str, shares: u32) -> PortfolioResult<()> {
        self.transact(symbol, shares, TransactionType::Sell)
    }

    fn transact(
        &mut self,
        symbol: &str,
        shares: u32,
        transaction_type: TransactionType,
    ) -> PortfolioResult<()> {
        Self::validate_share_count(shares)?;
        self.update_holdings(symbol, shares, transaction_type.clone())?;
        self.update_purchase_records(symbol, shares, transaction_type.clone())
    }

    fn update_holdings(
        &mut self,
        symbol: &str,
        shares: u32,
        transaction_type: TransactionType,
    ) -> PortfolioResult<()> {
        let count = self.holdings.entry(symbol.to_string()).or_default();
        let new_shares = match transaction_type {
            TransactionType::Purchase => count
                .checked_add(shares)
                .ok_or(PortfolioError::InvalidPurchase),

            TransactionType::Sell => count.checked_sub(shares).ok_or(PortfolioError::InvalidSell),
        }?;
        *count = new_shares;
        Ok(())
    }

    fn update_purchase_records(
        &mut self,
        symbol: &str,
        shares: u32,
        transaction_type: TransactionType,
    ) -> PortfolioResult<()> {
        let records = self.purchase_records.entry(symbol.to_string()).or_default();
        records.push(PurchaseRecord {
            date: Self::fixed_date_time(),
            shares,
            transaction_type,
        });
        Ok(())
    }

    fn share_count(&self, symbol: &str) -> u32 {
        *self.holdings.get(symbol).unwrap_or(&0)
    }

    fn get_purchase_record(&self, symbol: &str) -> PortfolioResult<&[PurchaseRecord]> {
        if let Some(records) = self.purchase_records.get(symbol) {
            Ok(records)
        } else {
            Err(PortfolioError::NoSymbolHistory)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    const IBM: &str = "IBM";
    const AAPL: &str = "AAPL";
    const UNPURCHASED_SYMBOL: &str = "unpurchased_symbol";

    #[fixture]
    fn portfolio() -> Portfolio {
        Portfolio::new()
    }

    #[fixture]
    fn portfolio_with_ibm() -> Portfolio {
        let mut p = Portfolio::new();
        p.purchase(IBM, 2).unwrap();
        p
    }

    #[rstest]
    fn empty_when_created(portfolio: Portfolio) {
        assert!(portfolio.is_empty());
    }

    #[rstest]
    fn not_empty_after_purchase(portfolio_with_ibm: Portfolio) -> PortfolioResult<()> {
        assert!(!portfolio_with_ibm.is_empty());
        Ok(())
    }

    #[rstest]
    fn answers_zero_for_share_count_of_unpurchased_symbol(portfolio: Portfolio) {
        assert_eq!(portfolio.share_count(UNPURCHASED_SYMBOL), 0);
    }

    #[rstest]
    fn answers_nonzero_for_share_count_of_purchased_symbol(portfolio_with_ibm: Portfolio) {
        assert!(portfolio_with_ibm.share_count(IBM) > 0);
    }

    #[rstest]
    fn cannot_purchase_zero_shares(mut portfolio: Portfolio) {
        assert!(matches!(
            portfolio.purchase(IBM, 0),
            Err(PortfolioError::ZeroShares)
        ));
    }

    #[rstest]
    fn answers_share_count_for_appropriate_symbol(
        mut portfolio_with_ibm: Portfolio,
    ) -> PortfolioResult<()> {
        let aapl_shares = 3;
        portfolio_with_ibm.purchase(AAPL, aapl_shares)?;
        assert_eq!(portfolio_with_ibm.share_count(AAPL), aapl_shares);
        Ok(())
    }

    #[rstest]
    fn share_count_reflects_accumulated_purchases_of_same_symbol(
        mut portfolio: Portfolio,
    ) -> PortfolioResult<()> {
        portfolio.purchase(IBM, 1)?;
        portfolio.purchase(IBM, 2)?;
        assert_eq!(portfolio.share_count(IBM), 3);
        Ok(())
    }

    #[rstest]
    fn reduce_share_count_of_symbol_on_sell(mut portfolio: Portfolio) -> PortfolioResult<()> {
        portfolio.purchase(IBM, 5)?;
        portfolio.sell(IBM, 3)?;
        assert_eq!(portfolio.share_count(IBM), 2);
        Ok(())
    }

    #[rstest]
    fn error_when_selling_more_shares_than_purchased(
        mut portfolio: Portfolio,
    ) -> PortfolioResult<()> {
        portfolio.purchase(IBM, 1)?;
        assert!(matches!(
            portfolio.sell(IBM, 2),
            Err(PortfolioError::InvalidSell)
        ));
        assert!(matches!(
            portfolio.sell(AAPL, 1),
            Err(PortfolioError::InvalidSell)
        ));
        Ok(())
    }

    #[rstest]
    fn error_when_selling_zero_shares(mut portfolio_with_ibm: Portfolio) {
        assert!(matches!(
            portfolio_with_ibm.sell(IBM, 0),
            Err(PortfolioError::ZeroShares)
        ));
    }

    #[rstest]
    fn answers_purchase_record_for_existing_share(mut portfolio: Portfolio) -> PortfolioResult<()> {
        let num_shares = 3u32;
        portfolio.purchase(IBM, num_shares)?;
        let record = portfolio.get_purchase_record(IBM)?;
        assert_eq!(
            record,
            vec![PurchaseRecord {
                date: Portfolio::fixed_date_time(),
                shares: num_shares,
                transaction_type: TransactionType::Purchase,
            }]
        );
        Ok(())
    }

    #[rstest]
    fn error_when_accessing_purchase_record_for_symbol_with_no_history(portfolio: Portfolio) {
        assert!(matches!(
            portfolio.get_purchase_record(IBM),
            Err(PortfolioError::NoSymbolHistory)
        ));
    }

    #[rstest]
    fn appends_purchase_record_when_purchasing_existing_share(
        mut portfolio_with_ibm: Portfolio,
    ) -> PortfolioResult<()> {
        portfolio_with_ibm.purchase(IBM, 10)?;
        let record = portfolio_with_ibm.get_purchase_record(IBM)?;
        assert_eq!(record.len(), 2);
        Ok(())
    }

    #[rstest]
    fn separates_purchase_records_by_symcol(mut portfolio: Portfolio) -> PortfolioResult<()> {
        let ibm_shares = 1;
        let aapl_shares = 2;
        portfolio.purchase(IBM, ibm_shares)?;
        portfolio.purchase(AAPL, aapl_shares)?;
        let aapl_shares_sell = aapl_shares - 1;
        portfolio.sell(AAPL, aapl_shares_sell)?;
        assert_eq!(
            portfolio.get_purchase_record(IBM)?,
            vec![PurchaseRecord {
                date: Portfolio::fixed_date_time(),
                shares: ibm_shares,
                transaction_type: TransactionType::Purchase
            }]
        );
        assert_eq!(
            portfolio.get_purchase_record(AAPL)?,
            vec![
                PurchaseRecord {
                    date: Portfolio::fixed_date_time(),
                    shares: aapl_shares,
                    transaction_type: TransactionType::Purchase
                },
                PurchaseRecord {
                    date: Portfolio::fixed_date_time(),
                    shares: aapl_shares_sell,
                    transaction_type: TransactionType::Sell
                }
            ]
        );
        Ok(())
    }
}
