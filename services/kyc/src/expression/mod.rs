mod evaluation;
mod node;
mod token;
pub mod traits;
pub mod types;

#[cfg(test)]
pub mod tests;

use derive_more::Display;
use protocol::types::Address;

use crate::expression::traits::ExpressionDataFeed;
use node::parse;
use token::scan;
use types::CalcContext;

#[derive(Debug, Display, PartialEq)]
pub enum ExpressionError {
    #[display(fmt = "scan token: {}", _0)]
    ScanError(String),

    #[display(fmt = "parse node: {}", _0)]
    ParseError(String),

    #[display(fmt = "calc node: {}", _0)]
    CalcError(String),
}

pub type ExpressionResult = Result<bool, ExpressionError>;

pub fn evaluate<DF: ExpressionDataFeed>(
    data_feeder: &DF,
    target_address: Address,
    expr: String,
) -> ExpressionResult {
    let tokens = scan(expr)?;
    let node = parse(tokens)?;
    let calc_context = CalcContext::new(data_feeder, target_address);
    calc_context.calculation(&node)
}

// s.t. regexp /^[a-zA-Z][a-zA-Z\d_]{0,11}}/
pub fn validate_ident_value(kyc_name: String) -> bool {
    if kyc_name.chars().count() > 12 {
        return false;
    }
    for (index, char) in kyc_name.chars().enumerate() {
        if !(char.is_ascii_alphanumeric() || char == '_') {
            return false;
        }
        if index == 0 && !char.is_ascii_alphabetic() {
            return false;
        }
    }
    true
}

// len between 1 to 6
// if there is NULL, the values can't contain any other values
pub fn validate_values_query(kyc_tag_values: Vec<String>) -> bool {
    let len = kyc_tag_values.len();

    if len == 0 || len > 6 {
        return false;
    }

    for value in kyc_tag_values {
        if !validate_ident_value(value.clone()) {
            return false;
        }

        if value.eq("NULL") && len != 1 {
            return false;
        }
    }

    true
}

// empty is not acceptable
// len > 6 is not acceptable
// can not contain NULL
#[allow(dead_code)]
pub fn validate_values_update(kyc_tag_values: Vec<String>) -> bool {
    let len = kyc_tag_values.len();

    if len == 0 || len > 6 {
        return false;
    }

    for value in kyc_tag_values {
        if !validate_ident_value(value.clone()) || value.eq("NULL") {
            return false;
        }
    }

    true
}
