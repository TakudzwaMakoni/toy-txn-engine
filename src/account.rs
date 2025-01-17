use std::collections::HashSet;

use crate::events::ProcessEvent;

#[derive(Debug, Clone)]
pub struct Account {
    pub available: u128,
    pub held: u128,
    pub disputes: HashSet<u32>,
    pub frozen: bool,
}

impl Account {
    pub fn new() -> Self {
        Account {
            available: 0,
            held: 0,
            // we are betting on the likelihood that
            // an account isnt going to have many disputes at one time
            // and also disputes get resolved fairly quickly
            // so this is rarely very large.
            disputes: HashSet::new(),
            frozen: false,
        }
    }

    pub fn add_available(&mut self, amount: u128) -> Result<(), ProcessEvent> {
        if let Some(new_balance) = self.available.checked_add(amount) {
            self.available = new_balance;
            Ok(())
        } else {
            Err(ProcessEvent::ExternalErr("limit exceeded".to_owned()))
        }
    }

    pub fn sub_available(&mut self, amount: u128) -> Result<(), ProcessEvent> {
        if let Some(new_balance) = self.available.checked_sub(amount) {
            self.available = new_balance;
            Ok(())
        } else {
            Err(ProcessEvent::ExternalErr("insufficient funds".to_owned()))
        }
    }

    pub fn add_held(&mut self, amount: u128) -> Result<(), ProcessEvent> {
        if let Some(new_balance) = self.held.checked_add(amount) {
            self.held = new_balance;
            Ok(())
        } else {
            Err(ProcessEvent::ExternalErr("limit exceeded".to_owned()))
        }
    }

    pub fn sub_held(&mut self, amount: u128) -> Result<(), ProcessEvent> {
        if let Some(new_balance) = self.held.checked_sub(amount) {
            self.held = new_balance;
            Ok(())
        } else {
            Err(ProcessEvent::ExternalErr("insufficient funds".to_owned()))
        }
    }

    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn total(&self) -> u128 {
        if let Some(total) = self.available.checked_add(self.held) {
            total
        } else {
            // handle deposit limit exceeded
            // (for now default to max value)
            u128::MAX
        }
    }
}
