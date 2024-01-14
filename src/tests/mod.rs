#[cfg(test)]
mod tests {
    use crate::*;
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
        assert_eq!(portfolio.get_share_count(UNPURCHASED_SYMBOL), 0);
    }

    #[rstest]
    fn answers_nonzero_for_share_count_of_purchased_symbol(portfolio_with_ibm: Portfolio) {
        assert!(portfolio_with_ibm.get_share_count(IBM) > 0);
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
        assert_eq!(portfolio_with_ibm.get_share_count(AAPL), aapl_shares);
        Ok(())
    }

    #[rstest]
    fn share_count_reflects_accumulated_purchases_of_same_symbol(
        mut portfolio: Portfolio,
    ) -> PortfolioResult<()> {
        portfolio.purchase(IBM, 1)?;
        portfolio.purchase(IBM, 2)?;
        assert_eq!(portfolio.get_share_count(IBM), 3);
        Ok(())
    }

    #[rstest]
    fn reduce_share_count_of_symbol_on_sell(mut portfolio: Portfolio) -> PortfolioResult<()> {
        portfolio.purchase(IBM, 5)?;
        portfolio.sell(IBM, 3)?;
        assert_eq!(portfolio.get_share_count(IBM), 2);
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
    fn separates_purchase_records_by_symbol(mut portfolio: Portfolio) -> PortfolioResult<()> {
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
