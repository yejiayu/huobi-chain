use protocol::types::Address;

use crate::expression::traits::ExpressionDataFeed;
use crate::expression::types::{CalcContext, CalcValue, Node, Token};
use crate::expression::{
    validate_ident_value, validate_values_query, ExpressionError, ExpressionResult,
};

pub struct CalcError(&'static str);

type CalcResult = Result<CalcValue, CalcError>;

impl From<CalcError> for ExpressionError {
    fn from(err: CalcError) -> ExpressionError {
        ExpressionError::CalcError(err.0.to_owned())
    }
}

impl<'a, DF: ExpressionDataFeed> CalcContext<'a, DF> {
    pub fn new(data_feeder: &'a DF, target_address: Address) -> Self {
        Self {
            data_feeder,
            target_address,
        }
    }

    pub fn calculation(&self, node: &Node) -> ExpressionResult {
        match self.calc(node)? {
            CalcValue::Bool(b) => Ok(b),
            _ => Err(CalcError("calculation result fails").into()),
        }
    }

    fn calc(&self, node: &Node) -> CalcResult {
        match node.token {
            Token::Dot => self.calc_dot(node),
            Token::Has => self.calc_has(node),
            Token::Not => self.calc_not(node),
            Token::And => self.calc_and(node),
            Token::Or => self.calc_or(node),
            Token::Value(_) => self.calc_value(node),
            Token::Identifier(_) => self.calc_ident(node),
            _ => unreachable!("wrong operation"),
        }
    }

    fn calc_ident(&self, ident_node: &Node) -> CalcResult {
        if let Token::Identifier(s) = &ident_node.token {
            Ok(CalcValue::Ident(s.to_owned()))
        } else {
            Err(CalcError("calc_ident get a wrong node"))
        }
    }

    fn calc_value(&self, value_node: &Node) -> CalcResult {
        if let Token::Value(s) = &value_node.token {
            Ok(CalcValue::Value(s.to_owned()))
        } else {
            Err(CalcError("calc_value get a wrong node"))
        }
    }

    fn calc_dot(&self, dot_node: &Node) -> CalcResult {
        let left = if let Some(kyc_node) = dot_node.left.as_ref() {
            match self.calc(kyc_node)? {
                CalcValue::Ident(str) => str,
                _ => return Err(CalcError("dot operation's left performs wrong")),
            }
        } else {
            return Err(CalcError("dot operation's left param is missing"));
        };

        let right = if let Some(tag_node) = dot_node.right.as_ref() {
            match self.calc(tag_node)? {
                CalcValue::Ident(str) => str,
                _ => return Err(CalcError("dot operation's right performs wrong")),
            }
        } else {
            return Err(CalcError("dot operation's right param is missing"));
        };

        // todo, calc KYC.TAG
        let tags = self
            .data_feeder
            .get_tags(self.target_address.clone(), left, right)
            .map_err(CalcError)?;

        Ok(CalcValue::KycTag(tags))
    }

    fn calc_has(&self, has_node: &Node) -> CalcResult {
        let kyc_tags = if let Some(kyc_tag_node) = has_node.left.as_ref() {
            match self.calc(kyc_tag_node)? {
                CalcValue::KycTag(values) => {
                    if !validate_values_query(values.clone()) {
                        return Err(CalcError("KYC.TAG's values is incorrect"));
                    }

                    values
                }
                _ => return Err(CalcError("has operation's left performs wrong")),
            }
        } else {
            return Err(CalcError("has operation's left param is missing"));
        };

        let value = if let Some(value_node) = has_node.right.as_ref() {
            match self.calc(value_node)? {
                CalcValue::Value(val) => {
                    if !validate_ident_value(val.clone()) {
                        return Err(CalcError("KYC.TAG's query value is incorrect"));
                    }

                    val
                }
                _ => return Err(CalcError("has operation's right performs wrong")),
            }
        } else {
            return Err(CalcError("has operation's right param is missing"));
        };

        for tag in kyc_tags {
            if tag == value {
                return Ok(CalcValue::Bool(true));
            }
        }

        Ok(CalcValue::Bool(false))
    }

    fn calc_not(&self, not_node: &Node) -> CalcResult {
        if not_node.left.as_ref().is_some() {
            return Err(CalcError("not operation's shouldn't have left param"));
        } else {
        }

        let right = if let Some(expr_node) = not_node.right.as_ref() {
            match self.calc(expr_node)? {
                CalcValue::Bool(b) => b,
                _ => return Err(CalcError("not operation's right performs wrong")),
            }
        } else {
            return Err(CalcError("not operation's right param is missing"));
        };

        Ok(CalcValue::Bool(!right))
    }

    fn calc_and(&self, and_node: &Node) -> CalcResult {
        let left = if let Some(expr_node) = and_node.left.as_ref() {
            match self.calc(expr_node)? {
                CalcValue::Bool(b) => b,
                _ => return Err(CalcError("and operation's left performs wrong")),
            }
        } else {
            return Err(CalcError("and operation's left param is missing"));
        };

        let right = if let Some(expr_node) = and_node.right.as_ref() {
            match self.calc(expr_node)? {
                CalcValue::Bool(b) => b,
                _ => return Err(CalcError("and operation's right performs wrong")),
            }
        } else {
            return Err(CalcError("and operation's right param is missing"));
        };

        Ok(CalcValue::Bool(left && right))
    }

    fn calc_or(&self, or_node: &Node) -> CalcResult {
        let left = if let Some(expr_node) = or_node.left.as_ref() {
            match self.calc(expr_node)? {
                CalcValue::Bool(b) => b,
                _ => return Err(CalcError("or operation's left performs wrong")),
            }
        } else {
            return Err(CalcError("or operation's left param is missing"));
        };

        let right = if let Some(expr_node) = or_node.right.as_ref() {
            match self.calc(expr_node)? {
                CalcValue::Bool(b) => b,
                _ => return Err(CalcError("or operation's right performs wrong")),
            }
        } else {
            return Err(CalcError("or operation's right param is missing"));
        };

        Ok(CalcValue::Bool(left || right))
    }
}
