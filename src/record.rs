use serde::{de::Error, Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(rename = "type")]
    pub r#type: String,
    pub client: u16,
    pub tx: u32,
    #[serde(default, deserialize_with = "amount_from_string")]
    pub amount: Option<u128>,
}

pub fn amount_from_string<'de, D>(deserializer: D) -> Result<Option<u128>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    };

    let processed = s.split('.').collect::<Vec<&str>>();

    // handle edge where int is supplied instead of
    // decimal.
    if processed.len() == 1 {
        let parsed = s.parse::<u128>();
        if parsed.is_err() {
            return Err(Error::custom("failed to parse decimal"));
        };

        return match parsed.unwrap().checked_mul(10000u128) {
            Some(val) => Ok(Some(val)),
            None => Err(Error::custom(format!(
                "failed to parse decimal: limit exceeded"
            ))),
        };
    }

    if let [before_point, after_point] = &processed[..] {
        let mut after_point_iter = after_point.chars();
        let char0 = after_point_iter.next().unwrap_or('0');
        let char1 = after_point_iter.next().unwrap_or('0');
        let char2 = after_point_iter.next().unwrap_or('0');
        let char3 = after_point_iter.next().unwrap_or('0');

        return match format!("{before_point}{char0}{char1}{char2}{char3}").parse::<u128>() {
            Ok(val) => Ok(Some(val)),
            Err(e) => Err(Error::custom(format!("failed to parse decimal: {e:?}"))),
        };
    }

    Err(Error::custom(String::from("failed to parse decimal")))
}

#[cfg(test)]
mod tests {
    use crate::record::Record;
    use serde_json;

    // we use serde_json instead of parsing a csv just for testing as
    // we can use a simple json string.

    #[test]
    fn test_custom_deserialise_record_amount() {
        let raw_string = r#"{ "type": "deposit", "client": 1, "tx":1, "amount": "1.5" }"#;
        let deserialised_record = serde_json::from_str::<Record>(raw_string);
        assert!(deserialised_record.is_ok());
        let record = deserialised_record.unwrap();

        // we want to check that the custom deserialisation
        // correctly denominated the amount.
        assert_eq!(record.amount, Some(15000));
        assert_eq!(record.client, 1u16);
        assert_eq!(record.tx, 1u32);
        assert_eq!(record.r#type, "deposit".to_owned());

        // now we try with the amount being only decimal
        let raw_string = r#"{ "type": "deposit", "client": 1, "tx":1, "amount": "0.1234" }"#;
        let record = serde_json::from_str::<Record>(raw_string).unwrap();
        assert_eq!(record.amount, Some(1234));

        // now we try with the amount being beyond 4 decimals
        // in which we choose to truncate after 4 decimals.
        let raw_string = r#"{ "type": "deposit", "client": 1, "tx":1, "amount": "0.123499999" }"#;
        let record = serde_json::from_str::<Record>(raw_string).unwrap();
        assert_eq!(record.amount, Some(1234));

        // now we try with the amount having no units before
        // the d.p. - which is sometimes considered valid.
        let raw_string = r#"{ "type": "deposit", "client": 1, "tx":1, "amount": ".0005" }"#;
        let record = serde_json::from_str::<Record>(raw_string).unwrap();
        assert_eq!(record.amount, Some(5));

        // now we try with an integer and not a decimal
        // to test that numbers in general are accepted
        let raw_string = r#"{ "type": "deposit", "client": 1, "tx":1, "amount": "100" }"#;
        let record = serde_json::from_str::<Record>(raw_string).unwrap();
        assert_eq!(record.amount, Some(100_0000));

        // now we try with a zero
        // to test that numbers in general are accepted
        let raw_string = r#"{ "type": "deposit", "client": 1, "tx":1, "amount": "0.0" }"#;
        let record = serde_json::from_str::<Record>(raw_string).unwrap();
        assert_eq!(record.amount, Some(0));

        // now we try with no amount supplied
        let raw_string = r#"{ "type": "dispute", "client": 1, "tx":1 }"#;
        let record = serde_json::from_str::<Record>(raw_string).unwrap();
        assert_eq!(record.amount, None)
    }
}
