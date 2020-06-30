use protocol::types::Address;

use crate::expression::{evaluate, traits::ExpressionDataFeed};

#[test]
pub fn test_parse() {
    let data_feeder = gen_data_feed();
    let address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    let ret = evaluate(&data_feeder, address.clone(), "X || Y`".to_string());
    assert_eq!(true, ret.is_err(), "unclosing_acute");

    let ret = evaluate(&data_feeder, address.clone(), "X&Y".to_string());
    assert_eq!(true, ret.is_err(), "single_logic_and 1");

    let ret = evaluate(&data_feeder, address.clone(), "X&&&Y&&".to_string());
    assert_eq!(true, ret.is_err(), "single_logic_and 2");

    let ret = evaluate(&data_feeder, address.clone(), "&X&&Y&&".to_string());
    assert_eq!(true, ret.is_err(), "single_logic_and 3");

    let ret = evaluate(&data_feeder, address.clone(), "X|Y".to_string());
    assert_eq!(true, ret.is_err(), "single_logic_or 1");

    let ret = evaluate(&data_feeder, address.clone(), "M||X Y|".to_string());
    assert_eq!(true, ret.is_err(), "single_logic_or 2");

    let ret = evaluate(&data_feeder, address.clone(), "X|Y|".to_string());
    assert_eq!(true, ret.is_err(), "single_logic_or 3");

    let ret = evaluate(&data_feeder, address.clone(), "( X @".to_string());
    assert_eq!(true, ret.is_err(), "unclosing_left_parenthesis 1");

    let ret = evaluate(&data_feeder, address.clone(), "( X (A@B) !".to_string());
    assert_eq!(true, ret.is_err(), "unclosing_left_parenthesis 2");

    let ret = evaluate(&data_feeder, address.clone(), "X @ )".to_string());
    assert_eq!(true, ret.is_err(), "unclosing_left_parenthesis 1");

    let ret = evaluate(&data_feeder, address.clone(), "X (A@B) )!".to_string());
    assert_eq!(true, ret.is_err(), "unclosing_left_parenthesis 2");

    let ret = evaluate(&data_feeder, address, "X () !".to_string());
    assert_eq!(true, ret.is_err(), "empty parenthesis");
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
            "Huobi.Nation@`UK`".to_string(),
        )
    );

    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "Lycrus.species@`Hamster`".to_string(),
        )
    );

    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "!Lycrus.species@`NULL`".to_string(),
        )
    );

    assert_eq!(
        Ok(false),
        evaluate(
            &data_feeder,
            address.clone(),
            "!Huobi.Nation@`UK` && Lycrus.species@`Hamster`".to_string(),
        )
    );

    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "Lycrus.planet@`NULL`".to_string(),
        )
    );

    // Die Abenteuer der Maus auf dem Mars
    // Lycrus is a hamster indeed, but not comes from Mars
    assert_eq!(
        Ok(false),
        evaluate(
            &data_feeder,
            address.clone(),
            "Lycrus.planet@`Mars`".to_string(),
        )
    );

    assert_eq!(
        Ok(true),
        evaluate(
            &data_feeder,
            address.clone(),
            "(!Huobi.Nation@`UK` || Lycrus.species@`Hamster`) && Lycrus.planet@`NULL`".to_string(),
        )
    );

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!.Nation@`UK` || Lycrus.species@`Hamster`) && Lycrus.planet@`NULL`".to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_left_dot 1");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`UK` || .species@`Hamster`) && Lycrus.planet@`NULL`".to_string(),
    );
    // println!("empty_left_dot 2 : {:?}", ret);
    assert_eq!(true, ret.is_err(), "empty_left_dot 2");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`UK` || Lycrus.species@`Hamster`) && .planet@`NULL`".to_string(),
    );
    // println!("empty_left_dot 3 : {:?}", ret);
    assert_eq!(true, ret.is_err(), "empty_left_dot 3");

    //======

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.@`UK` || Lycrus.species@`Hamster`) && Lycrus.planet@`NULL`".to_string(),
    );
    // println!("empty_right_dot 1 : {:?}", ret);
    assert_eq!(true, ret.is_err(), "empty_right_dot 1");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`UK` || Lycrus.@`Hamster`) && Lycrus.planet@`NULL`".to_string(),
    );
    // println!("empty_right_dot 2 : {:?}", ret);
    assert_eq!(true, ret.is_err(), "empty_right_dot 2");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`UK` || Lycrus.planet@`Hamster`) && Lycrus.@`NULL`".to_string(),
    );
    // println!("empty_right_dot 3 : {:?}", ret);
    assert_eq!(true, ret.is_err(), "empty_right_dot 3");

    //======

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@ || Lycrus.species@`Hamster`) && Lycrus.planet@`NULL`".to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_has 1");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`UK` || Lycrus.species@) && Lycrus.planet@`NULL`".to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_has 2");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`UK` || Lycrus.species@`Hamster`) && Lycrus.planet@".to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_has 3");

    //======

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`Hamster` ||) && Lycrus.planet@`NULL`".to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_logic_or");

    //======

    let ret = evaluate(
        &data_feeder,
        address,
        "(!Huobi.Nation@`Hamster` || Lycrus.species@`Hamster` ) && ".to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_logic_and");
}

#[test]
pub fn test_ident_format() {
    let data_feeder = gen_data_feed();
    let address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!1Huobi.Nation@`Hamster` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`"
            .to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_logic_and 1");

    let ret = evaluate(&data_feeder, address.clone(),
                       "(!HuobiHuobiHuobiHuobiHuobiHuobiHuobiHuobiHuobiHuobi.Nation@`Hamster` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`".to_string());
    assert_eq!(true, ret.is_err(), "empty_logic_and 2");

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.2Nation@`Hamster` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`"
            .to_string(),
    );
    assert_eq!(true, ret.is_err(), "empty_logic_and 3");

    let ret = evaluate(&data_feeder, address.clone(),
                       "(!Huobi.NationNationNationNationNationNationNationNationNationNationNationNationNation@`Hamster` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`".to_string());
    assert_eq!(true, ret.is_err(), "empty_logic_and 4");

    //======

    let ret = evaluate(
        &data_feeder,
        address.clone(),
        "(!Huobi.Nation@`` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`".to_string(),
    );
    assert_eq!(true, ret.is_err(), "wrong_value 1");

    let ret = evaluate(&data_feeder, address.clone(),
                       "(!Huobi.Nation@`HamsterHamsterHamsterHamsterHamsterHamsterHamsterHamsterHamsterHamsterHamster` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`".to_string());
    assert_eq!(true, ret.is_err(), "wrong_value 2");

    let ret = evaluate(
        &data_feeder,
        address,
        "(!Huobi.Nation@`1Hamster` || Lycrus.species@`Hamster` ) && Lycrus.planet@`NULL`"
            .to_string(),
    );
    assert_eq!(true, ret.is_err(), "wrong_value 3");
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
