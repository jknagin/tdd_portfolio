mod tests;
use chrono::NaiveDateTime;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransactionType {
    Purchase,
    Sell,
}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq)]
pub struct PurchaseRecord {
    pub date: NaiveDateTime,
    pub shares: u32,
    pub transaction_type: TransactionType,
}

pub struct Portfolio {
    holdings: HashMap<String, u32>,
    purchase_records: HashMap<String, Vec<PurchaseRecord>>,
}

#[derive(Debug, thiserror::Error)]
pub enum PortfolioError {
    #[error("Cannot perform transaction with zero shares")]
    ZeroShares,

    #[error("Cannot sell more shares than owned")]
    InvalidSell,

    #[error("No history for symbol")]
    NoSymbolHistory,

    #[error("Too many shares puchased")]
    InvalidPurchase,
}

pub type PortfolioResult<T> = Result<T, PortfolioError>;

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)]
impl Portfolio {
    const FIXED_EPOCH_TIME_MS: i64 = 0;

    const EMPTY_PURCHASE_RECORD: Vec<PurchaseRecord> = vec![];

    pub fn fixed_date_time() -> NaiveDateTime {
        NaiveDateTime::from_timestamp_millis(Self::FIXED_EPOCH_TIME_MS).unwrap()
    }

    pub fn new() -> Self {
        Self {
            holdings: HashMap::new(),
            purchase_records: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.holdings.is_empty()
    }

    fn validate_share_count(shares: u32) -> PortfolioResult<()> {
        if shares == 0 {
            return Err(PortfolioError::ZeroShares);
        }
        Ok(())
    }

    pub fn purchase(&mut self, symbol: &str, shares: u32) -> PortfolioResult<()> {
        self.transact(symbol, shares, TransactionType::Purchase)
    }

    pub fn sell(&mut self, symbol: &str, shares: u32) -> PortfolioResult<()> {
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

    pub fn get_share_count(&self, symbol: &str) -> u32 {
        *self.holdings.get(symbol).unwrap_or(&0)
    }

    pub fn get_purchase_record(&self, symbol: &str) -> PortfolioResult<&[PurchaseRecord]> {
        self.purchase_records
            .get(symbol)
            .map(|x| x.as_slice())
            .ok_or(PortfolioError::NoSymbolHistory)
    }
}
