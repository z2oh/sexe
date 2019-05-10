#[macro_use]
extern crate nom;
extern crate sexe_expression;

use nom::types::CompleteStr;
use sexe_expression::*;
use std::f64::consts::{E, PI};

// This is a custom implementation of nom::recognize_float that does not parse
// the optional sign before the number, so that expressions like `x+3` parse
// correctly and not as `x(+3)`.
named!(recognize_float<CompleteStr, CompleteStr>,
    recognize!(
        tuple!(
            alt!(
                value!((), tuple!(nom::digit, opt!(pair!(char!('.'), opt!(nom::digit))))) |
                value!((), tuple!(char!('.'), nom::digit))
            ),
            opt!(tuple!(alt!(char!('e') | char!('E')), nom::digit))
        )
    )
);

/// Helper macro for defining simple unary functions to be invoked with function
/// call like syntax (like `sin(x)`). The first argument is the name of the
/// function, (e.g. `parse_sin`), the second argument is the UnaryOperator
/// expression node type, (e.g. `UnaryOperator::Sin`), and then the remaining
/// arguments are a comma separated list of different valid parse strings for
/// this function (e.g. `"asin", "arcsin").
macro_rules! def_unary_fn_parser {
    ($name:ident, $op:expr, $($strs:expr),+) => (
        named!($name<CompleteStr, ExpressionNode>,
            do_parse!(
                // N.B. We have `take!(0)` as a noop parser because nom does not
                // allow for trailing `|` chars in `alt!` combinators.
                alt!($(tag!($strs)|)+take!(0)) >>
                res: parse_parens >>
                (ExpressionNode::UnaryExprNode {
                    operator: $op,
                    child_node: Box::new(res),
                })
            )
        );
    );
}

named!(parse_double<CompleteStr, f64>,
    flat_map!(recognize_float, parse_to!(f64))
);

named!(parse_constant<CompleteStr, ExpressionNode>,
    do_parse!(
        value: parse_double >>
        (ExpressionNode::ConstantExprNode { value, })
    )
);

named!(parse_variable<CompleteStr, ExpressionNode>,
    do_parse!(
        var: take_while1!(|x: char| x.is_alphabetic()) >>
        (ExpressionNode::VariableExprNode { variable_key: var.to_string(), })
    )
);

named!(parse_coefficient<CompleteStr, ExpressionNode>,
    do_parse!(
        coefficient: parse_priority_1 >>
        res: parse_priority_1 >>
        (ExpressionNode::BinaryExprNode {
            operator: BinaryOperator::Multiplication,
            left_node: Box::new(coefficient),
            right_node: Box::new(res),
        })
    )
);

named!(parse_parens<CompleteStr, ExpressionNode>,
    ws!(delimited!(char!('('), parse_expr, char!(')')))
);

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

named!(parse_args<CompleteStr, Vec<ExpressionNode>>,
    delimited!(char!('('), separated_list!(tag!(","), parse_expr), char!(')'))
);

named!(parse_log<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("log") >>
        res: parse_args >>
        (ExpressionNode::NaryExprNode {
            operator: NaryOperator::Log,
            child_nodes: Box::new(res),
        })
    )
);

named!(parse_e<CompleteStr, ExpressionNode>,
    do_parse!(
        tag_no_case!("e") >>
        // TODO: Can we make this lazier (i.e. stop parsing after one character
        // matches)?
        // Ensure this constant is not followed by any other characters.
        not!(call!(nom::alpha1)) >>
        (ExpressionNode::ConstantExprNode { value: E, })
    )
);

named!(parse_pi<CompleteStr, ExpressionNode>,
    do_parse!(
        alt!(tag_no_case!("pi") | tag!("π")) >>
        // TODO: Can we make this lazier (i.e. stop parsing after one character
        // matches)?
        // Ensure this constant is not followed by any other characters.
        not!(call!(nom::alpha1)) >>
        (ExpressionNode::ConstantExprNode { value: PI })
    )
);

named!(parse_abs_bar_syntax<CompleteStr, ExpressionNode>,
    do_parse!(
        res: delimited!(char!('|'), parse_expr, char!('|')) >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Abs,
            child_node: Box::new(res),
        })
    )
);

named!(parse_expr<CompleteStr, ExpressionNode>,
    call!(parse_priority_4)
);

named!(parse_priority_0<CompleteStr, ExpressionNode>,
    // TODO: Figure out a way to avoid redefining these if a parser is already
    // defined using the `def_unary_fn_parser!` macro?
    ws!(alt_complete!(
        parse_constant       |
        parse_parens         |
        parse_sin            |
        parse_asin           |
        parse_cos            |
        parse_acos           |
        parse_tan            |
        parse_ctan           |
        parse_abs            |
        parse_exp            |
        parse_log2           |
        parse_log10          |
        parse_ln             |
        parse_ceil           |
        parse_floor          |
        parse_abs_bar_syntax |
        parse_log            |
        // N.B. These must go after the other parsers, or e.g. parse_e will
        // match `exp(x)`.
        parse_e              |
        parse_pi             |
        parse_variable
    ))
);

named!(parse_priority_1<CompleteStr, ExpressionNode>,
    do_parse!(
        init: parse_priority_0 >>
        res: fold_many0!(
            ws!(pair!(alt!(tag!("^")), parse_priority_0)),
            init,
            |acc, (op, val): (CompleteStr, ExpressionNode)| {
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
        ) >>
        (res)
    )
);

named!(parse_priority_2<CompleteStr, ExpressionNode>,
    do_parse!(
        init: alt!(
            parse_coefficient |
            parse_priority_1
        ) >>
        res: fold_many0!(
            ws!(pair!(alt!(tag!("*") | tag!("/")), parse_priority_1)),
            init,
            |acc, (op, val): (CompleteStr, ExpressionNode)| {
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
        ) >>
        (res)
    )
);

named!(parse_priority_3<CompleteStr, ExpressionNode>,
    alt_complete!(
        do_parse!(
            op: alt!(tag!("-")) >>
            res: parse_priority_2 >>
            (ExpressionNode::UnaryExprNode {
                operator: match op.as_bytes()[0] as char {
                    '-' => UnaryOperator::Negation,
                    // For now, default to Negation.
                    _ => UnaryOperator::Negation,
                },
                child_node: Box::new(res),
            })
        ) |
        parse_priority_2
    )
);

named!(parse_priority_4<CompleteStr, ExpressionNode>,
    do_parse!(
        init: parse_priority_3 >>
        res: fold_many0!(
            ws!(pair!(alt!(tag!("+") | tag!("-")), parse_priority_3)),
            init,
            |acc, (op, val): (CompleteStr, ExpressionNode)| {
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
        ) >>
        (res)
    )
);

pub fn parse(function_string: &str) -> Result<ExpressionNode, ()> {
    if let Ok((rem, func)) = parse_expr(CompleteStr(function_string)) {
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
                parse_expr(CompleteStr($inp))
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
                parse_expr(CompleteStr($inp))
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
                parse_expr(CompleteStr($inp))
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
                parse_expr(CompleteStr($inp))
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
        eval_test!("log(9,3)", 2.0, &vars_map);
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
