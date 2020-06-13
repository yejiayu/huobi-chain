use crate::KycService;
use std::cell::RefCell;
use std::sync::Arc;
use std::rc::Rc;

use framework::binding::sdk::{DefalutServiceSDK, DefaultChainQuerier};
use framework::binding::state::{GeneralServiceState, MPTTrie};
use framework::executor::ServiceExecutor;
use protocol::traits::{
    Context, Executor, ExecutorParams, NoopDispatcher, Service, ServiceMapping, ServiceSDK, Storage,
};
use protocol::types::{Address};

use crate::expression::{evaluate, traits::ExpressionDataFeed};
use crate::expression::types::KYCError;

#[test]
pub fn test_parse(){
    let data_feeder = gen_data_feed();
    let address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();

    let a =  evaluate(&data_feeder, address.clone(), "X || Y`".to_string());
    println!("{:?}", a);

}

#[test]
pub fn test_valuation() {
    let data_feeder = gen_data_feed();
    let address = Address::from_hex("0x755cdba6ae4f479f7164792b318b2a06c759833b").unwrap();
    assert_eq!(Ok(true), evaluate(&data_feeder, address.clone(), "Huobi.Nation@`UK`".to_string()));

    assert_eq!(Ok(true), evaluate(&data_feeder, address.clone(), "Lycrus.species@`Hamster`".to_string()));

    assert_eq!(Ok(true), evaluate(&data_feeder, address.clone(), "!Lycrus.species@`NULL`".to_string()));

    assert_eq!(Ok(false), evaluate(&data_feeder, address.clone(), "!Huobi.Nation@`UK` && Lycrus.species@`Hamster`".to_string()));
}


pub struct DefaultDataFeed {}

impl ExpressionDataFeed for DefaultDataFeed {
    fn get_tags(&self, target_address: Address, kyc: String, tag: String) -> Vec<String> {
        if kyc == "Huobi" && tag == "Nation" {
            return vec!["US".to_string(), "UK".to_string()];
        }
        if kyc == "Lycrus" && tag == "species" {
            return vec!["Hamster".to_string(), "Rodent".to_string()];
        }
        vec!["NULL".to_string()]
    }
}

pub fn gen_data_feed() -> DefaultDataFeed {
    DefaultDataFeed {}
}