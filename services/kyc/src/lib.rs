pub mod expression;

use binding_macro::service;
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK};
use protocol::types::{Address, ServiceContext};

use expression::{evaluate, traits::ExpressionDataFeed};

#[derive(Debug)]
pub struct KycService<SDK> {
    sdk: SDK,
}

#[service]
impl<SDK: ServiceSDK> KycService<SDK> {
    pub fn new(sdk: SDK) -> Self {
        Self { sdk }
    }

    #[allow(dead_code)]
    fn eval(&self, target_address: Address, expr: String) {
        if evaluate(self, target_address, expr).is_ok() {
            println!("OK");
        };
    }
}

impl<SDK: ServiceSDK> ExpressionDataFeed for KycService<SDK> {
    fn get_tags(&self, target_address: Address, kyc: String, tag: String) -> Vec<String> {
        println!("get_tags:{}:{}.{}", target_address.as_hex(), kyc, tag);
        vec!["KYC.TAG".to_string()]
    }
}
