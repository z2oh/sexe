use expression::*;
use nom;
use nom::types::CompleteStr;

pub fn parse_double(input: CompleteStr) -> nom::IResult<CompleteStr, f64> {
    flat_map!(input, call!(nom::recognize_float), parse_to!(f64))
}

pub fn parse_constant(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    do_parse!(
        input,
        value: parse_double >> (ExpressionNode::ConstantExprNode { value: value })
    )
}

pub fn parse_exponentiation(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    do_parse!(
        input,
        left_node: take_until!("^") >> tag!("^") >> right_node: take_while!(|x| true)
            >> (ExpressionNode::BinaryExprNode {
                operator: BinaryOperator::Exponentiation,
                left_node: Box::new(parse_expr(left_node).unwrap().1),
                right_node: Box::new(parse_expr(right_node).unwrap().1),
            })
    )
}

pub fn parse_multiplication(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    do_parse!(
        input,
        left_node: take_until!("*") >> tag!("*") >> right_node: take_while!(|x| true)
            >> (ExpressionNode::BinaryExprNode {
                operator: BinaryOperator::Multiplication,
                left_node: Box::new(parse_expr(left_node).unwrap().1),
                right_node: Box::new(parse_expr(right_node).unwrap().1),
            })
    )
}

pub fn parse_division(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    do_parse!(
        input,
        left_node: take_until!("/") >> tag!("/") >> right_node: take_while!(|x| true)
            >> (ExpressionNode::BinaryExprNode {
                operator: BinaryOperator::Division,
                left_node: Box::new(parse_expr(left_node).unwrap().1),
                right_node: Box::new(parse_expr(right_node).unwrap().1),
            })
    )
}

pub fn parse_addition(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    do_parse!(
        input,
        left_node: take_until!("+") >> tag!("+") >> right_node: take_while!(|x| true)
            >> (ExpressionNode::BinaryExprNode {
                operator: BinaryOperator::Addition,
                left_node: Box::new(parse_expr(left_node).unwrap().1),
                right_node: Box::new(parse_expr(right_node).unwrap().1),
            })
    )
}

pub fn parse_subtraction(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    do_parse!(
        input,
        left_node: take_until!("-") >> tag!("-") >> right_node: take_while!(|x| true)
            >> (ExpressionNode::BinaryExprNode {
                operator: BinaryOperator::Subtraction,
                left_node: Box::new(parse_expr(left_node).unwrap().1),
                right_node: Box::new(parse_expr(right_node).unwrap().1),
            })
    )
}

pub fn parse_binary(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    alt_complete!(
        input,
        parse_addition | parse_subtraction | parse_multiplication | parse_division
            | parse_exponentiation
    )
}

fn parse_expr(input: CompleteStr) -> nom::IResult<CompleteStr, ExpressionNode> {
    alt_complete!(input, parse_binary | parse_constant)
}

#[test]
fn test_parse_expr() {
    assert_eq!(
        parse_expr(CompleteStr("2*7^2")).unwrap().1.evaluate(&[]),
        98.0
    );

    assert_eq!(
        parse_expr(CompleteStr("49-7^2")).unwrap().1.evaluate(&[]),
        0.0
    );

    assert_eq!(
        parse_expr(CompleteStr("10-2+4")).unwrap().1.evaluate(&[]),
        12.0
    );

    assert_eq!(
        parse_expr(CompleteStr("50/2*10")).unwrap().1.evaluate(&[]),
        250.0
    );
}
