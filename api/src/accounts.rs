//! Current accounts of the (fictional) demo customers, held in memory.
//!
//! This stands in for the deposits system of record. Amounts are in EUR, rounded to cents.

use serde::Serialize;
use std::collections::BTreeMap;

pub fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Transaction {
    pub transaction_id: String,
    pub amount: f64,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Account {
    pub account_id: String,
    pub customer_id: String,
    pub product: String,
    pub currency: String,
    pub balance: f64,
    pub transactions: Vec<Transaction>,
}

/// All accounts, keyed by account id.
#[derive(Debug, Clone)]
pub struct Ledger {
    pub accounts: BTreeMap<String, Account>,
}

impl Ledger {
    pub fn demo() -> Self {
        let mut accounts = BTreeMap::new();
        for (id, customer, balance) in [("CA-2001", "CUST-1001", 1_250.00), ("CA-2002", "CUST-1002", 310.40), ("CA-2003", "CUST-1003", 2_045.75)] {
            accounts.insert(id.to_string(), Account {
                account_id: id.into(),
                customer_id: customer.into(),
                product: "Current account".into(),
                currency: "EUR".into(),
                balance,
                transactions: vec![],
            });
        }
        Ledger { accounts }
    }

    pub fn get(&self, account_id: &str) -> Option<&Account> {
        self.accounts.get(account_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_accounts_exist() {
        let l = Ledger::demo();
        assert_eq!(l.get("CA-2001").unwrap().balance, 1_250.00);
        assert!(l.get("CA-9999").is_none());
    }
}
