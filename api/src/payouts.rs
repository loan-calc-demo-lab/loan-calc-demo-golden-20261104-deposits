//! REQ-008: payouts into a current account (contract deposits-payouts-v1, POST /payouts).
//!
//! Money movement is idempotent (architecture standard ARC-4): the caller's `Idempotency-Key` is stored with the
//! result. A repeat with the same key and body returns the first result and credits nothing; the same key with a
//! different body is a conflict. Only Deposits changes a balance (ARC-6).

use crate::accounts::{round2, Ledger, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct PayoutRequest {
    pub account_id: String,
    pub amount: f64,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Payout {
    pub payout_id: String,
    pub account_id: String,
    pub amount: f64,
    pub reference: String,
    pub new_balance: f64,
    pub created_at: String,
}

#[derive(Debug, PartialEq)]
pub enum PayoutOutcome {
    /// A new payout: the account was credited (HTTP 201).
    Created(Payout),
    /// A repeat of an earlier request with the same key and body: nothing was credited (HTTP 200).
    Repeated(Payout),
}

#[derive(Debug, PartialEq)]
pub enum PayoutError {
    UnknownAccount,
    /// The key was already used with a different request.
    KeyConflict,
    InvalidAmount,
}

/// Payouts made so far, by idempotency key.
#[derive(Debug, Default)]
pub struct PayoutBook {
    by_key: HashMap<String, (PayoutRequest, Payout)>,
    count: u64,
}

impl PayoutBook {
    /// REQ-008 AC5: credit `request.amount` to the account, at most once per idempotency key.
    pub fn pay(&mut self, ledger: &mut Ledger, key: &str, request: PayoutRequest, now: &str) -> Result<PayoutOutcome, PayoutError> {
        if let Some((first_request, first_result)) = self.by_key.get(key) {
            return if *first_request == request {
                Ok(PayoutOutcome::Repeated(first_result.clone()))
            } else {
                Err(PayoutError::KeyConflict)
            };
        }
        if !request.amount.is_finite() || request.amount <= 0.0 {
            return Err(PayoutError::InvalidAmount);
        }
        let account = ledger.accounts.get_mut(&request.account_id).ok_or(PayoutError::UnknownAccount)?;
        self.count += 1;
        let payout_id = payout_id(self.count);
        let amount = round2(request.amount);
        account.balance = round2(account.balance + amount);
        account.transactions.push(Transaction { transaction_id: payout_id.clone(), amount, reference: request.reference.clone() });
        let payout = Payout {
            payout_id,
            account_id: request.account_id.clone(),
            amount,
            reference: request.reference.clone(),
            new_balance: account.balance,
            created_at: now.to_string(),
        };
        self.by_key.insert(key.to_string(), (request, payout.clone()));
        Ok(PayoutOutcome::Created(payout))
    }
}

/// A unique id in the 8-4-4-4-12 hex format (no extra crates).
fn payout_id(n: u64) -> String {
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    let hex = format!("{:016x}{:016x}", nanos as u64, n);
    format!("{}-{}-{}-{}-{}", &hex[0..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..32])
}

/// ISO 8601 UTC time for Unix seconds (no extra crates).
pub fn iso_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req(account: &str, amount: f64) -> PayoutRequest {
        PayoutRequest { account_id: account.into(), amount, reference: "topup-PL-1001".into() }
    }

    #[test]
    fn req008_ac5_credits_once_per_key() {
        let mut ledger = Ledger::demo();
        let mut book = PayoutBook::default();
        let first = book.pay(&mut ledger, "k", req("CA-2001", 4950.0), "t").unwrap();
        let again = book.pay(&mut ledger, "k", req("CA-2001", 4950.0), "t").unwrap();
        assert!(matches!(first, PayoutOutcome::Created(ref p) if p.new_balance == 6200.0));
        assert!(matches!(again, PayoutOutcome::Repeated(ref p) if p.new_balance == 6200.0));
        assert_eq!(ledger.get("CA-2001").unwrap().balance, 6200.0);
    }

    #[test]
    fn iso_utc_formats_a_known_time() {
        assert_eq!(iso_utc(1_793_750_400), "2026-11-04T00:00:00Z");
    }
}
