extern crate nom;
extern crate sexe_expression;

use std::f64::consts::{E, PI};

use nom::IResult;
use nom::ParseTo;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case};
use nom::character::complete::{alpha1, char};
use nom::combinator::not;
use nom::multi::separated_list0;
use nom::sequence::{delimited, pair};

use sexe_expression::*;

mod custom_combinators;
use crate::custom_combinators::{recognize_float, fold_many0_once, ws};


/// Helper macro for defining simple unary functions to be invoked with function
/// call like syntax (like `sin(x)`). The first argument is the name of the
/// function, (e.g. `parse_sin`), the second argument is the UnaryOperator
/// expression node type, (e.g. `UnaryOperator::Sin`), and then the remaining
/// arguments are a comma separated list of different valid parse strings for
/// this function (e.g. `"asin", "arcsin").
macro_rules! def_unary_fn_parser {
    // When only parse string is provided, we cannot use an alt combinator, so
    // we parse the string directly with tag.
    ($name:ident, $op:expr, $str:expr) => (
        fn $name(i: &str) -> IResult<&str, ExpressionNode> {
            let (i, _) = tag($str)(i)?;
            let (i, res) = parse_parens(i)?;
            Ok((i, ExpressionNode::UnaryExprNode {
                operator: $op,
                child_node: Box::new(res),
            }))
        }
    );
    // If multiple parse strings are provided, we wrap them in an alt
    // combinator.
    ($name:ident, $op:expr, $($strs:expr),+) => (
        fn $name(i: &str) -> IResult<&str, ExpressionNode> {
            let (i, _) = alt(($(tag($strs),)+))(i)?;
            let (i, res) = parse_parens(i)?;
            Ok((i, ExpressionNode::UnaryExprNode {
                operator: $op,
                child_node: Box::new(res),
            }))
        }
    );
}

fn parse_double(i: &str) -> IResult<&str, f64> {
    let (i, f) = recognize_float(i)?;
    Ok((i, f.parse_to().unwrap()))
}

fn parse_constant(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, value) = parse_double(i)?;
    Ok((i, ExpressionNode::ConstantExprNode { value, }))
}

fn parse_variable(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, var) = alpha1(i)?;
    Ok((i, ExpressionNode::VariableExprNode { variable_key: var.to_string(), }))
}

fn parse_coefficient(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, coefficient) = parse_priority_1(i)?;
    let (i, res) = parse_priority_1(i)?;
    Ok((i, ExpressionNode::BinaryExprNode {
        operator: BinaryOperator::Multiplication,
        left_node: Box::new(coefficient),
        right_node: Box::new(res),
    }))
}

fn parse_parens(i: &str) -> IResult<&str, ExpressionNode> {
    ws(delimited(char('('), parse_expr, char(')')))(i)
}

def_unary_fn_parser!(parse_sin, UnaryOperator::Sin, "sin");
def_unary_fn_parser!(parse_asin, UnaryOperator::Asin, "asin", "arcsin");
def_unary_fn_parser!(parse_cos, UnaryOperator::Cos, "cos");
def_unary_fn_parser!(parse_acos, UnaryOperator::Acos, "acos", "arccos");
def_unary_fn_parser!(parse_tan, UnaryOperator::Tan, "tan", "tg");
def_unary_fn_parser!(parse_ctan, UnaryOperator::Ctan, "ctan", "ctg");
def_unary_fn_parser!(parse_abs, UnaryOperator::Abs, "abs");
def_unary_fn_parser!(parse_log2, UnaryOperator::Log2, "log2");
def_unary_fn_parser!(parse_log10, UnaryOperator::Log10, "log10");
def_unary_fn_parser!(parse_ln, UnaryOperator::Ln, "ln");
def_unary_fn_parser!(parse_exp, UnaryOperator::Exp, "exp");
def_unary_fn_parser!(parse_ceil, UnaryOperator::Ceil, "ceil");
def_unary_fn_parser!(parse_floor, UnaryOperator::Floor, "floor");

fn parse_args(i: &str) -> IResult<&str, Vec<ExpressionNode>> {
    //let (i, _) = char('(')(i)?;
    //let (i, res) = separated_list(tag(","), parse_expr)(i)?;
    //let (i, _) = char(')')(i)?;
    //Ok((i, res))
    delimited(char('('), separated_list0(tag(","), parse_expr), char(')'))(i)
}

fn parse_log(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, _) = tag("log")(i)?;
    let (i, res) = parse_args(i)?;
    Ok((i, ExpressionNode::NaryExprNode {
        operator: NaryOperator::Log,
        child_nodes: Box::new(res),
    }))
}

fn parse_e(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, _) = alt((char('e'), char('E')))(i)?;
    not(alpha1)(i)?;
    Ok((i, ExpressionNode::ConstantExprNode { value: E, }))
}

fn parse_pi(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, _) = alt((tag_no_case("pi"), tag("π")))(i)?;
    not(alpha1)(i)?;
    Ok((i, ExpressionNode::ConstantExprNode { value: PI, }))
}

fn parse_abs_bar_syntax(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, res) = delimited(char('|'), parse_expr, char('|'))(i)?;
    Ok((i, ExpressionNode::UnaryExprNode {
        operator: UnaryOperator::Abs,
        child_node: Box::new(res),
    }))
}

fn parse_expr(i: &str) -> IResult<&str, ExpressionNode> {
    parse_priority_4(i)
}

fn parse_priority_0(i: &str) -> IResult<&str, ExpressionNode> {
    // TODO: Figure out a way to avoid redefining these if a parser is already
    // defined using the `def_unary_fn_parser!` macro?
    ws(alt((
        parse_constant,
        parse_parens,
        parse_sin,
        parse_asin,
        parse_cos,
        parse_acos,
        parse_tan,
        parse_ctan,
        parse_abs,
        parse_exp,
        parse_log2,
        parse_log10,
        parse_ln,
        parse_ceil,
        parse_floor,
        parse_abs_bar_syntax,
        parse_log,
        // N.B. These must go after the other parsers, or e.g. parse_e will
        // match `exp(x)`.
        parse_e,
        parse_pi,
        parse_variable
    )))(i)
}

fn parse_priority_1(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, init) = parse_priority_0(i)?;
    fold_many0_once(
        |i: &str| { ws(pair(tag("^"), parse_priority_0))(i) },
        init,
        |acc, (op, val): (&str, ExpressionNode)| {
            let operator = match op.as_bytes()[0] as char {
                '^' => BinaryOperator::Exponentiation,
                // For now, default to Exponentiation.
                _ => BinaryOperator::Exponentiation,
            };
            ExpressionNode::BinaryExprNode {
                operator,
                left_node: Box::new(acc),
                right_node: Box::new(val),
            }
        }
    )(i)
}

fn parse_priority_2(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, init) = alt((parse_coefficient, parse_priority_1))(i)?;
    fold_many0_once(
        |i: &str| { ws(pair(alt((tag("*"), tag("/"))), parse_priority_1))(i) },
        init,
        |acc, (op, val): (&str, ExpressionNode)| {
            let operator = match op.as_bytes()[0] as char {
                '*' => BinaryOperator::Multiplication,
                '/' => BinaryOperator::Division,
                // For now, default to Multiplication.
                _   => BinaryOperator::Multiplication,
            };
            ExpressionNode::BinaryExprNode {
                operator,
                left_node: Box::new(acc),
                right_node: Box::new(val),
            }
        }
    )(i)
}

fn parse_priority_3(i: &str) -> IResult<&str, ExpressionNode> {
    fn _parse_priority_3_internal(i: &str) -> IResult<&str, ExpressionNode> {
        let (i, op) = tag("-")(i)?;
        let (i, res) = parse_priority_2(i)?;
        Ok((i, ExpressionNode::UnaryExprNode {
            operator: match op.as_bytes()[0] as char {
                '-' => UnaryOperator::Negation,
                // For now, default to Negation.
                _ => UnaryOperator::Negation,
            },
            child_node: Box::new(res),
        }))
    }

    alt((_parse_priority_3_internal, parse_priority_2))(i)
}

fn parse_priority_4(i: &str) -> IResult<&str, ExpressionNode> {
    let (i, init) = parse_priority_3(i)?;
    fold_many0_once(
        |i: &str| { ws(pair(alt((tag("+"), tag("-"))), parse_priority_3))(i) },
        init,
        |acc, (op, val): (&str, ExpressionNode)| {
            let operator = match op.as_bytes()[0] as char {
                '+' => BinaryOperator::Addition,
                '-' => BinaryOperator::Subtraction,
                // For now, default to Addition.
                _   => BinaryOperator::Addition,
            };
            ExpressionNode::BinaryExprNode {
                operator,
                left_node: Box::new(acc),
                right_node: Box::new(val),
            }
        }
    )(i)
}

pub fn parse(function_string: &str) -> Result<ExpressionNode, ()> {
    if let Ok((rem, func)) = parse_expr(function_string) {
        // Make sure we consumed the entire input.
        if rem.len() > 0 {
            Err(())
        }
        else {
            Ok(func)
        }
    }
    else {
        Err(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::collections::HashMap;

    macro_rules! eval_test {
        // Use the specified variable map.
        ($inp:expr, $out:expr, $vars:expr) => {
            assert_eq!(
                parse_expr($inp)
                    .unwrap()
                    .1
                    .evaluate($vars)
                    .unwrap(),
                $out
            );
        };
        // Assume an empty variable map.
        ($inp:expr, $out:expr) => {
            assert_eq!(
                parse_expr($inp)
                    .unwrap()
                    .1
                    .evaluate(&HashMap::new())
                    .unwrap(),
                $out
            );
        };
    }

    macro_rules! error_test {
        // Use the specified variable map.
        ($inp:expr, $err:expr, $vars:expr) => {
            assert_eq!(
                parse_expr($inp)
                    .unwrap()
                    .1
                    .evaluate($vars)
                    .err()
                    .unwrap(),
                $err
            );
        };
        // Assume an empty variable map.
        ($inp:expr, $err:expr) => {
            assert_eq!(
                parse_expr($inp)
                    .unwrap()
                    .1
                    .evaluate(&HashMap::new())
                    .err()
                    .unwrap(),
                $err
            );
        };
    }

    #[test]
    fn trivial_expressions() {
        let mut vars_map = HashMap::new();
        vars_map.insert("x".to_string(), 10.0);

        eval_test!("3", 3.0);
        eval_test!("x", 10.0, &vars_map);
    }

    #[test]
    fn constant_expression() {
        // Constant expression parsing
        eval_test!("(3(3))", 9.0);
        eval_test!("3(3(3))", 27.0);
        eval_test!("3+10", 13.0);
        eval_test!("3-(2+1)", 0.0);
        eval_test!("3-(2-1)", 2.0);
        eval_test!("3-(2-3+1)+(4-1+4)", 10.0);
        eval_test!("3+2+2-8+1-3", -3.0);
        eval_test!("3-4-5-6", -12.0);
        eval_test!("2*2/(5-1)+3", 4.0);
        eval_test!("2/2/(5-1)*3", 0.75);
        eval_test!("-4*4", -16.0);
        eval_test!("3*(-3)", -9.0);
    }

    #[test]
    fn variable_expressions() {
        let mut vars_map = HashMap::new();
        vars_map.insert("x".to_string(), 10.0);

        eval_test!("3(x(3))", 90.0, &vars_map);
        eval_test!("3x", 30.0, &vars_map);
        eval_test!("-x*sin(0)", 0.0, &vars_map);
        eval_test!("3^3", 27.0, &vars_map);
        eval_test!("2^3", 8.0, &vars_map);
        eval_test!("3^2", 9.0, &vars_map);
        eval_test!("3^(-3)", 1.0 / 27.0, &vars_map);
        eval_test!("(((2(4)))))", 8.0, &vars_map);
        eval_test!("-2^4", -16.0, &vars_map);
        eval_test!("(-2)^4", 16.0, &vars_map);
        eval_test!("exp(0)", 1.0, &vars_map);
        eval_test!("log2(2)", 1.0, &vars_map);
        eval_test!("log2(8)", 3.0, &vars_map);
        eval_test!("log(9,3)", 2.0);
        eval_test!("3 -   (2  -  3 + 1   ) + (  4 - 1    +4 )", 10.0, &vars_map);
        eval_test!("ln(e)", 1.0, &vars_map);
        eval_test!("sin (   0   )", 0.0, &vars_map);
        eval_test!("sin (   0 * pi  )", 0.0, &vars_map);
        eval_test!("log( 9 , 3)", 2.0, &vars_map);
    }

    #[test]
    fn error_tests() {
        let mut vars_map = HashMap::new();
        vars_map.insert("x".to_string(), 10.0);
        vars_map.insert("foo".to_string(), 10.0);

        error_test!("log(3,9,5)", EvaluationError::WrongNumberOfArgsError);
        error_test!("log(3,    9   ,5)", EvaluationError::WrongNumberOfArgsError);
        error_test!("y", EvaluationError::VariableNotFoundError, &vars_map);
    }
}
