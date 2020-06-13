pub mod types;
pub mod traits;
mod token;
mod node;
mod evaluation;


#[cfg(test)]
pub mod tests;

use std::error::Error;
use derive_more::Display;
use std::collections::{HashMap, VecDeque};

use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK};
use protocol::types::{Address};


use super::KycService;
use types::{ExpressionResult,CalcContext};
use token::scan;
use node::parse;
use crate::expression::traits::ExpressionDataFeed;

pub fn evaluate<DF: ExpressionDataFeed>(data_feeder:& DF, target_address:Address, expr:String) -> ExpressionResult{
    let tokens = scan(expr)?;
    let node = parse(tokens)?;
    let calc_context = CalcContext::new(data_feeder,target_address);
    calc_context.calculation(&node)

}