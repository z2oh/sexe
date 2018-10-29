use expression::*;
use nom;
use nom::types::CompleteStr;
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

named!(parse_sin<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("sin") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Sin,
            child_node: Box::new(res),
        })
    )
);

named!(parse_cos<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("cos") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Cos,
            child_node: Box::new(res),
        })
    )
);

named!(parse_tan<CompleteStr, ExpressionNode>,
    do_parse!(
        alt!(tag!("tan") | tag!("tg")) >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Tan,
            child_node: Box::new(res),
        })
    )
);

named!(parse_ctan<CompleteStr, ExpressionNode>,
    do_parse!(
        alt_complete!(tag!("ctan") | tag!("ctg")) >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Ctan,
            child_node: Box::new(res),
        })
    )
);

named!(parse_abs<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("abs") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Abs,
            child_node: Box::new(res),
        })
    )
);

named!(parse_log2<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("log2") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Log2,
            child_node: Box::new(res),
        })
    )
);

named!(parse_ln<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("ln") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Ln,
            child_node: Box::new(res),
        })
    )
);

named!(parse_exp<CompleteStr, ExpressionNode>,
    do_parse!(
        tag!("exp") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Exp,
            child_node: Box::new(res),
        })
    )
);

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

named!(parse_acos<CompleteStr, ExpressionNode>,
    do_parse!(
        alt!(tag!("acos") | tag!("arccos")) >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Acos,
            child_node: Box::new(res),
        })
    )
);

named!(parse_asin<CompleteStr, ExpressionNode>,
    do_parse!(
        alt!(tag!("asin") | tag!("arcsin")) >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Asin,
            child_node: Box::new(res),
        })
    )
);

named!(parse_e<CompleteStr, ExpressionNode>,
    do_parse!(
        beg: tag_no_case!("e") >>
        rest: take_while!(|x: char| x.is_alphabetic())  >>
        (
            match rest.len() {
                0 => ExpressionNode::ConstantExprNode { value: E },
                _ => ExpressionNode::VariableExprNode { variable_key: format!("{}{}", beg, rest) }
            }
        )
    )
);

named!(parse_pi<CompleteStr, ExpressionNode>,
    do_parse!(
        beg: alt!(tag_no_case!("pi") | tag!("π")) >>
        rest: take_while!(|x: char| x.is_alphabetic()) >>
        (
            match rest.len() {
                0 => ExpressionNode::ConstantExprNode { value: PI },
                _ => ExpressionNode::VariableExprNode { variable_key: format!("{}{}", beg, rest) }
            }
        )
    )
);

named!(parse_modulus<CompleteStr, ExpressionNode>,
    do_parse!(
        res: delimited!(char!('|'), parse_expr, char!('|')) >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Abs,
            child_node: Box::new(res),
        })
    )
);


named!(pub parse_expr<CompleteStr, ExpressionNode>,
    call!(parse_priority_4)
);

named!(parse_priority_0<CompleteStr, ExpressionNode>,
    ws!(alt_complete!(
        parse_constant   |
        parse_parens     |
        parse_sin        |
        parse_cos        |
        parse_tan        |
        parse_ctan       |
        parse_asin       |
        parse_acos       |
        parse_abs        |
        parse_modulus    |
        parse_log2       |
        parse_ln         |
        parse_exp        |
        parse_log        |
        parse_e          |
        parse_pi         |
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
