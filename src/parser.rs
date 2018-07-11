use expression::*;
use nom;
use nom::types::CompleteStr;

named!(parse_double<CompleteStr, f64>,
    flat_map!(call!(nom::recognize_float), parse_to!(f64))
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
    call!(parse_term)
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
}
