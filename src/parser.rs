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
    delimited!( char!('('), parse_expr, char!(')') )
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
        tag!("tan") >>
        res: parse_parens >>
        (ExpressionNode::UnaryExprNode {
            operator: UnaryOperator::Tan,
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

named!(pub parse_expr<CompleteStr, ExpressionNode>,
    call!(parse_priority_4)
);

named!(parse_priority_0<CompleteStr, ExpressionNode>,
    alt_complete!(
        parse_constant   |
        parse_parens     |
        parse_sin        |
        parse_cos        |
        parse_tan        |
        parse_abs        |
        parse_exp        |
        parse_variable
    )
);

named!(parse_priority_1<CompleteStr, ExpressionNode>,
    do_parse!(
        init: parse_priority_0 >>
        res: fold_many0!(
            pair!(alt!(tag!("^")), parse_priority_0),
            init,
            |acc, (op, val): (CompleteStr, ExpressionNode)| {
                let operator = match op.as_bytes()[0] as char {
                    '^' => BinaryOperator::Exponentiation,
                    // For now, default to Exponentiatino.
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
            pair!(alt!(tag!("*") | tag!("/")), parse_priority_1),
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
            pair!(alt!(tag!("+") | tag!("-")), parse_priority_3),
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

#[test]
fn test_parse_constant() {
    use std::collections::HashMap;
    assert_eq!(
        parse_constant(CompleteStr("3"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        3.0
    );

    let mut vars_map = HashMap::new();
    vars_map.insert("x".to_string(), 10.0);

    assert_eq!(
        parse_variable(CompleteStr("x"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        10.0
    );
}

#[test]
fn test_parse_term() {
    use std::collections::HashMap;
    let mut vars_map = HashMap::new();
    vars_map.insert("x".to_string(), 10.0);

    assert_eq!(
        parse_expr(CompleteStr("(3(3))"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        9.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3x"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        30.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3(3(3))"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        27.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3(x(3))"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        90.0
    );

    assert_eq!(
        parse_expr(CompleteStr("3+10"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        13.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3-(2+1)"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        0.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3-(2-1)"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        2.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3-(2-3+1)+(4-1+4)"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        10.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3+2+2-8+1-3"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        -3.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3-4-5-6"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        -12.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("2*2/(5-1)+3"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        4.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("2/2/(5-1)*3"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        0.75,
    );

    assert_eq!(
        parse_expr(CompleteStr("-4*4"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        -16.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3*(-3)"))
            .unwrap()
            .1
            .evaluate(&HashMap::new())
            .unwrap(),
        -9.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("-x*sin(0)"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        0.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3^3"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        27.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("2^3"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        8.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3^2"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        9.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("3^(-3)"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        1.0 / 27.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("(((2(4)))))"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        8.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("-2^4"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        -16.0,
    );

    assert_eq!(
        parse_expr(CompleteStr("(-2)^4"))
            .unwrap()
            .1
            .evaluate(&vars_map)
            .unwrap(),
        16.0,
    );
}
