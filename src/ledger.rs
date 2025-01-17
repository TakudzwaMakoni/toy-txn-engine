use std::collections::HashMap;

use crate::{account::Account, events::ProcessEvent, record::Record, transaction::Txn};

pub struct Ledger {
    pub accounts: HashMap<u16, Account>,
    pub txn_history: HashMap<u32, Txn>,
}

impl Ledger {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            txn_history: HashMap::new(),
        }
    }

    fn txn_from_history(&self, txn_id: u32) -> Option<&Txn> {
        self.txn_history.get(&txn_id)
    }
    /// Deposit to available balance.
    ///
    /// Will fail if available balance exceeds u128::MAX.
    ///
    /// Will fail if the account is frozen.
    ///
    /// If the deposit fails the app will
    /// continue to process other transactions.
    fn deposit(&mut self, txn: Txn) -> Result<(), ProcessEvent> {
        let account = self
            .accounts
            .entry(txn.client_id())
            .or_insert(Account::new());

        if account.frozen {
            // handle account frozen
        } else if account.add_available(txn.amount()).is_err() {
            // handle deposit failed here
        }

        self.txn_history.insert(txn.txn_id(), txn);
        Ok(())
    }

    /// Withdraw from available balance.
    ///
    /// Will fail if available balance is insufficient.
    ///
    /// Will fail if the account is frozen.
    ///
    /// If the withdrawal fails the app will
    /// continue to process other transactions.
    fn withdraw(&mut self, txn: Txn) -> Result<(), ProcessEvent> {
        let account = self
            .accounts
            .entry(txn.client_id())
            .or_insert(Account::new());

        if account.frozen {
            // handle account frozen
        } else if account.sub_available(txn.amount()).is_err() {
            // handle withdrawal failed here
        }

        self.txn_history.insert(txn.txn_id(), txn);
        Ok(())
    }

    /// dispute a referenced transaction.
    ///
    /// If referenced txn does not exist will ignore.
    fn dispute(&mut self, txn: &Txn) -> Result<(), ProcessEvent> {
        let txn_id = txn.txn_id();
        // assume partner error if txn referenced
        // does not exist and ignore.
        if let Some(referenced_txn) = self.txn_from_history(txn_id) {

            // only valid for deposits, ignore otherwise
            if matches!(referenced_txn, Txn::Deposit { .. }) {
                let amount = referenced_txn.amount();
                let account = self
                    .accounts
                    .entry(referenced_txn.client_id())
                    .or_insert(Account::new());
    
                account.sub_available(amount)?;
                account.add_held(amount)?;
                account.disputes.insert(txn_id);
            }
        }
        Ok(())
    }

    /// resolve a referenced transaction.
    ///
    /// If referenced txn does not exist will ignore.
    ///
    /// If referenced is not in dispute will ignore.
    fn resolve(&mut self, txn: &Txn) -> Result<(), ProcessEvent> {
        let txn_id = txn.txn_id();

        // assume partner error if txn referenced
        // does not exist, or txn not disputed and ignore.
        if let Some(referenced_txn) = self.txn_from_history(txn_id) {
            let amount = referenced_txn.amount();
            let account = self
                .accounts
                .entry(referenced_txn.client_id())
                .or_insert(Account::new());

            if account.disputes.contains(&txn_id) {
                account.sub_held(amount)?;
                account.add_available(amount)?;
                account.disputes.remove(&txn_id);
            }
        }
        Ok(())
    }

    /// chargeback a referenced transaction.
    ///
    /// If referenced txn does not exist will ignore.
    ///
    /// If referenced is not in dispute will ignore.
    fn chargeback(&mut self, txn: &Txn) -> Result<(), ProcessEvent> {
        let txn_id = txn.txn_id();

        // assume partner error if txn referenced
        // does not exist, or txn not disputed and ignore.
        if let Some(referenced_txn) = self.txn_from_history(txn_id) {
            let amount = referenced_txn.amount();
            let account = self
                .accounts
                .entry(referenced_txn.client_id())
                .or_insert(Account::new());

            if account.disputes.contains(&txn_id) {
                account.sub_held(amount)?;
                account.disputes.remove(&txn_id);
                account.freeze();
            }
        }

        Ok(())
    }

    fn add_tx_to_account(&mut self, txn: Txn) -> Result<(), ProcessEvent> {
        match txn {
            Txn::Deposit { .. } => self.deposit(txn)?,
            Txn::Withdraw { .. } => self.withdraw(txn)?,
            Txn::Dispute { .. } => self.dispute(&txn)?,
            Txn::Resolve { .. } => self.resolve(&txn)?,
            Txn::ChargeBack { .. } => self.chargeback(&txn)?,
        }
        Ok(())
    }

    pub fn process_transaction(&mut self, record: Record) -> Result<ProcessEvent, ProcessEvent> {
        let txn = Txn::from_record(record)?;
        self.add_tx_to_account(txn)?;
        Ok(ProcessEvent::ProcessComplete)
    }

    pub fn print_accounts(&self) -> Result<(), ProcessEvent> {
        println!(
            "{: >10},{: >10},{: >10},{: >10},{: >10}",
            "client", "available", "held", "total", "locked"
        );
        for (key, val) in self.accounts.iter() {
            let available = Txn::u128_to_decimal_str(val.available)?;
            let held = Txn::u128_to_decimal_str(val.held)?;
            let total = Txn::u128_to_decimal_str(val.total())?;
            let frozen = val.frozen;
            println!(
                "{: >10},{: >10},{: >10},{: >10},{: >10}",
                key, available, held, total, frozen
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{account::Account, events::ProcessEvent, ledger::Record};

    use super::Ledger;

    fn record(r#type: String, client: u16, tx: u32, amount: Option<u128>) -> Record {
        Record {
            r#type,
            client,
            tx,
            amount,
        }
    }

    #[test]
    fn test_deposit() -> Result<(), ProcessEvent> {
        let mut ledger = Ledger::new();

        ledger.process_transaction(record("deposit".to_owned(), 1, 1, Some(5_0000)))?;
        ledger.process_transaction(record("deposit".to_owned(), 1, 2, Some(5)))?;
        ledger.process_transaction(record("deposit".to_owned(), 2, 3, Some(270_0000)))?;
        ledger.process_transaction(record("deposit".to_owned(), 2, 4, Some(1234)))?;

        let account1: &Account = ledger.accounts.get(&1).unwrap();
        let account2: &Account = ledger.accounts.get(&2).unwrap();

        assert_eq!(account1.available, 5_0005);
        assert_eq!(account2.available, 270_1234);

        Ok(())
    }

    #[test]
    fn test_withdrawal() -> Result<(), ProcessEvent> {
        let mut ledger = Ledger::new();

        ledger.process_transaction(record("deposit".to_owned(), 1, 1, Some(1000_0000)))?;
        ledger.process_transaction(record("withdrawal".to_owned(), 1, 2, Some(700_0000)))?;
        ledger.process_transaction(record("deposit".to_owned(), 2, 3, Some(10_0000)))?;
        ledger.process_transaction(record("withdrawal".to_owned(), 2, 4, Some(100_0000)))?;

        let account1: &Account = ledger.accounts.get(&1).unwrap();
        let account2: &Account = ledger.accounts.get(&2).unwrap();

        assert_eq!(account1.available, 300_0000); // withdrawal succeeded
        assert_eq!(account2.available, 10_0000); // withdrawal failed

        Ok(())
    }

    #[test]
    fn test_dispute() -> Result<(), ProcessEvent> {
        let mut ledger = Ledger::new();

        ledger.process_transaction(record("deposit".to_owned(), 1, 1, Some(1000_0000)))?;
        ledger.process_transaction(record("deposit".to_owned(), 1, 2, Some(700_0000)))?;
        ledger.process_transaction(record("dispute".to_owned(), 1, 2, None))?;

        let account: &Account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.available, 1000_0000);
        assert_eq!(account.held, 700_0000);

        // let client 2 dispute client 1's txn #1
        ledger.process_transaction(record("dispute".to_owned(), 2, 1, None))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.available, 0);
        assert_eq!(account.held, 1700_0000);
        Ok(())
    }

    #[test]
    fn test_resolve() -> Result<(), ProcessEvent> {
        let mut ledger = Ledger::new();

        ledger.process_transaction(record("deposit".to_owned(), 1, 1, Some(1000_0000)))?;
        ledger.process_transaction(record("deposit".to_owned(), 1, 2, Some(700_0000)))?;
        ledger.process_transaction(record("dispute".to_owned(), 1, 2, None))?;
        ledger.process_transaction(record("resolve".to_owned(), 1, 2, None))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.available, 1700_0000);
        assert_eq!(account.held, 0);

        // try resolve undisputed txn #1
        ledger.process_transaction(record("resolve".to_owned(), 1, 1, None))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        // confirm its ignored
        assert_eq!(account.available, 1700_0000);
        assert_eq!(account.held, 0);

        Ok(())
    }

    #[test]
    fn test_chargeback() -> Result<(), ProcessEvent> {
        let mut ledger = Ledger::new();

        ledger.process_transaction(record("deposit".to_owned(), 1, 1, Some(1000_0000)))?;
        ledger.process_transaction(record("deposit".to_owned(), 1, 2, Some(700_0000)))?;
        ledger.process_transaction(record("dispute".to_owned(), 1, 2, None))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.available, 1000_0000);
        assert_eq!(account.held, 700_0000);
        assert!(!account.frozen);

        ledger.process_transaction(record("chargeback".to_owned(), 1, 2, None))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        assert_eq!(account.available, 1000_0000);
        assert_eq!(account.held, 0);
        assert!(account.frozen);

        // try a deposit
        ledger.process_transaction(record("deposit".to_owned(), 1, 3, Some(1000_0000)))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        // funds the same but account frozen
        assert_eq!(account.available, 1000_0000);
        assert_eq!(account.held, 0);
        assert!(account.frozen);

        // try a deposit
        ledger.process_transaction(record("withdrawal".to_owned(), 1, 4, Some(100_0000)))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        // state is the same
        assert_eq!(account.available, 1000_0000);
        assert_eq!(account.held, 0);
        assert!(account.frozen);

        //try to chargeback undisputed
        ledger.process_transaction(record("chargeback".to_owned(), 1, 1, None))?;
        let account: &Account = ledger.accounts.get(&1).unwrap();

        // nothing changes
        assert_eq!(account.available, 1000_0000);
        assert_eq!(account.held, 0);
        assert!(account.frozen);

        Ok(())
    }
}
