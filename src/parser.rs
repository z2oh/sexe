use expression::*;
use nom;
use nom::types::CompleteStr;

// This is a custom implementation of nom::recognize_float that does not parse
// the optional sign before the number, so that expressions like `x+3` parse
// correctly and not as `x(+3)`.
named!(recognize_float<CompleteStr, CompleteStr>,
  recognize!(
    tuple!(
      alt!(
        value!((), tuple!(nom::digit, opt!(pair!(char!('.'), opt!(nom::digit)))))
      | value!((), tuple!(char!('.'), nom::digit))
      ),
      opt!(tuple!(
        alt!(char!('e') | char!('E')),
        nom::digit
        )
      )
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
        var: take_while1!(|x| nom::is_alphabetic(x as u8)) >>
        (ExpressionNode::VariableExprNode { variable_key: var.to_string(), })
    )
);

named!(parse_constant_coefficient<CompleteStr, ExpressionNode>,
    do_parse!(
        coefficient: parse_constant >>
        term: parse_term >>
        (ExpressionNode::BinaryExprNode {
            operator: BinaryOperator::Multiplication,
            left_node: Box::new(coefficient),
            right_node: Box::new(term),
        })
    )
);

named!(parse_variable_coefficient<CompleteStr, ExpressionNode>,
    do_parse!(
        coefficient: parse_variable >>
        term: parse_term >>
        (ExpressionNode::BinaryExprNode {
            operator: BinaryOperator::Multiplication,
            left_node: Box::new(coefficient),
            right_node: Box::new(term),
        })
    )
);

named!(parse_coefficient<CompleteStr, ExpressionNode>,
    alt_complete!(
        parse_constant_coefficient |
        parse_variable_coefficient
    )
);

named!(parse_parens<CompleteStr, ExpressionNode>,
    delimited!( char!('('), parse_expr, char!(')') )
);

named!(parse_expr<CompleteStr, ExpressionNode>,
    do_parse!(
        init: parse_term >>
        res:  fold_many0!(
            pair!(alt!(tag!("+") | tag!("-")), parse_expr),
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

named!(parse_term<CompleteStr, ExpressionNode>,
    alt_complete!(
        parse_coefficient |
        parse_parens      |
        parse_variable    |
        parse_constant
    )
);

use std::collections::HashMap;
#[test]
fn test_parse_constant() {
    assert_eq!(
        parse_constant(CompleteStr("3")).unwrap().1.evaluate(&HashMap::new()),
        3.0
    );

    let mut vars_map = HashMap::new();
    vars_map.insert("x".to_string(), 10.0);

    assert_eq!(
        parse_variable(CompleteStr("x")).unwrap().1.evaluate(&vars_map),
        10.0
    );
}

#[test]
fn test_parse_term() {
    let mut vars_map = HashMap::new();
    vars_map.insert("x".to_string(), 10.0);

    assert_eq!(
        parse_expr(CompleteStr("(3(3))")).unwrap().1.evaluate(&HashMap::new()),
        9.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3x")).unwrap().1.evaluate(&vars_map),
        30.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3(3(3))")).unwrap().1.evaluate(&HashMap::new()),
        27.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3(x(3))")).unwrap().1.evaluate(&vars_map),
        90.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3+10")).unwrap().1.evaluate(&HashMap::new()),
        13.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3-(2+1)")).unwrap().1.evaluate(&HashMap::new()),
        0.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3-(2-1)")).unwrap().1.evaluate(&HashMap::new()),
        2.0,
    );
}
