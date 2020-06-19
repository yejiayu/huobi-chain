use protocol::types::Address;

use crate::expression::{evaluate, traits::ExpressionDataFeed};

#[test]
pub fn test_parse() {
    let data_feeder = gen_data_feed();
    let address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    let a = evaluate(&data_feeder, address, "X || Y`".to_string());
    println!("{:?}", a);
}

#[test]
pub fn test_valuation() {
    let data_feeder = gen_data_feed();
    let address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "Huobi.Nation@`UK`".to_string()
        )
    );

    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "Lycrus.species@`Hamster`".to_string()
        )
    );

    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "!Lycrus.species@`NULL`".to_string()
        )
    );

    assert_eq!(
        Ok(false),
        evaluate(
            &data_feeder,
            address,
            "!Huobi.Nation@`UK` && Lycrus.species@`Hamster`".to_string()
        )
    );
}

pub struct DefaultDataFeed {}

impl ExpressionDataFeed for DefaultDataFeed {
    fn get_tags(
        &self,
        _target_address: Address,
        kyc: String,
        tag: String,
    ) -> Result<Vec<String>, &'static str> {
        if kyc == "Huobi" && tag == "Nation" {
            return Ok(vec!["US".to_string(), "UK".to_string()]);
        }

        if kyc == "Lycrus" && tag == "species" {
            return Ok(vec!["Hamster".to_string(), "Rodent".to_string()]);
        }

        Ok(vec!["NULL".to_string()])
    }
}

pub fn gen_data_feed() -> DefaultDataFeed {
    DefaultDataFeed {}
}
