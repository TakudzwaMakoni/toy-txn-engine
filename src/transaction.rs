use crate::{events::ProcessEvent, record::Record};

#[derive(PartialEq)]
pub enum Txn {
    Deposit {
        client_id: u16,
        txn_id: u32,
        amount: u128,
    },
    Withdraw {
        client_id: u16,
        txn_id: u32,
        amount: u128,
    },
    Dispute {
        client_id: u16,
        txn_id: u32,
    },
    Resolve {
        client_id: u16,
        txn_id: u32,
    },
    ChargeBack {
        client_id: u16,
        txn_id: u32,
    },
}

impl Txn {
    /// transform deserialised decimal back to string format
    /// with 4 decimals.
    pub fn u128_to_decimal_str(input: u128) -> Result<String, ProcessEvent> {
        let as_str = format!("{:0>4}", input);
        let [_units, decimals] = {
            // unwrap will not panic as is
            // guaranteed by padding by 4 on previous line.
            let split_pos = as_str.char_indices().nth_back(3).unwrap().0;
            [&as_str[..split_pos], &as_str[split_pos..]]
        };

        let units = if _units.is_empty() { "0" } else { _units };
        Ok(format!("{units}.{decimals}"))
    }

    pub fn client_id(&self) -> u16 {
        match self {
            Self::Deposit { client_id, .. } => *client_id,
            Self::Withdraw { client_id, .. } => *client_id,
            Self::Dispute { client_id, .. } => *client_id,
            Self::Resolve { client_id, .. } => *client_id,
            Self::ChargeBack { client_id, .. } => *client_id,
        }
    }

    pub fn txn_id(&self) -> u32 {
        match self {
            Self::Deposit { txn_id, .. } => *txn_id,
            Self::Withdraw { txn_id, .. } => *txn_id,
            Self::Dispute { txn_id, .. } => *txn_id,
            Self::Resolve { txn_id, .. } => *txn_id,
            Self::ChargeBack { txn_id, .. } => *txn_id,
        }
    }

    pub fn amount(&self) -> u128 {
        match self {
            Self::Deposit { amount, .. } => *amount,
            Self::Withdraw { amount, .. } => *amount,
            _ => 0u128,
        }
    }

    pub fn from_record(input: Record) -> Result<Self, ProcessEvent> {
        let txn_type = input.r#type;
        let client_id = input.client;
        let txn_id = input.tx;

        match txn_type.as_str() {
            "deposit" => {
                if let Some(amount) = input.amount {
                    Ok(Self::Deposit {
                        client_id,
                        txn_id,
                        amount,
                    })
                } else {
                    return Err(ProcessEvent::ExternalErr(
                        "deposit needs an amount".to_owned(),
                    ));
                }
            }
            "withdrawal" => {
                if let Some(amount) = input.amount {
                    Ok(Self::Withdraw {
                        client_id,
                        txn_id,
                        amount,
                    })
                } else {
                    return Err(ProcessEvent::ExternalErr(
                        "withdrawal needs an amount".to_owned(),
                    ));
                }
            }
            "dispute" => Ok(Self::Dispute { client_id, txn_id }),
            "resolve" => Ok(Self::Resolve { client_id, txn_id }),
            "chargeback" => Ok(Self::ChargeBack { client_id, txn_id }),
            _ => Err(ProcessEvent::ExternalErr(format!(
                "unrecognised txn: {txn_type}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::transaction::Txn;

    #[test]
    fn test_u128_to_decimal_string() {
        let value = 12345;
        let to_string = Txn::u128_to_decimal_str(value);
        assert_eq!(to_string, Ok(String::from("1.2345")));

        let value = 100_2345;
        let to_string = Txn::u128_to_decimal_str(value);
        assert_eq!(to_string, Ok(String::from("100.2345")));

        let value = 2345;
        let to_string = Txn::u128_to_decimal_str(value);
        assert_eq!(to_string, Ok(String::from("0.2345")));

        let value = 5;
        let to_string = Txn::u128_to_decimal_str(value);
        assert_eq!(to_string, Ok(String::from("0.0005")));

        let value = 0;
        let to_string = Txn::u128_to_decimal_str(value);
        assert_eq!(to_string, Ok(String::from("0.0000")));
    }
}
