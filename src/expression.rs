/// These are the supported binary operators.
#[derive(Debug)]
pub enum BinaryOperator {
    /// Addition: `+`
    Addition,
    /// Subtraction: `-`
    Subtraction,
    /// Multiplication: `*`
    Multiplication,
    /// Division: `/`
    Division,
    /// Exponentiation: `^`
    Exponentiation,
}

/// These are the supported unary operators.
#[derive(Debug)]
pub enum UnaryOperator {
    /// Sin: `sin()`
    Sin,
    /// Cos: `cos()`
    Cos,
    /// Negation: `-`, as in `-4`
    Negation,
}

/// An expression node is any part of the parsed expression tree. These build up the expression
/// recursively. Every value, variable, and operator is wrapped in an `ExpressionNode`.
#[derive(Debug)]
pub enum ExpressionNode {
    /// This variant holds an operator that is to be applied to the evaluated values of its left
    /// and right subtrees of the expression.
    BinaryExprNode {
        operator: BinaryOperator,
        left_node: Box<ExpressionNode>,
        right_node: Box<ExpressionNode>,
    },
    /// This variant holds an operator that is to be applied to the evaluated value of its child
    /// subtree of the expression.
    UnaryExprNode {
        operator: UnaryOperator,
        child_node: Box<ExpressionNode>,
    },
    /// This variant holds an index into the `vars` array which indicates which variable of the
    /// expression it represents.
    VariableExprNode { variable_index: usize },
    /// This variant holds a constant value.
    ConstantExprNode { value: f64 },
}

impl ExpressionNode {
    /// Takes in an array of variables to recursively pass down to all `ExpressionNode`s until the
    /// expression is evaluated. The `f64` value returned is the result of the expression tree
    /// rooted at `self`.
    pub fn evaluate(&self, vars: &[f64]) -> f64 {
        match self {
            ExpressionNode::BinaryExprNode {
                operator,
                left_node,
                right_node,
            } => {
                let left_value = left_node.evaluate(&vars);
                let right_value = right_node.evaluate(&vars);
                match operator {
                    BinaryOperator::Addition => left_value + right_value,
                    BinaryOperator::Subtraction => left_value - right_value,
                    BinaryOperator::Multiplication => left_value * right_value,
                    BinaryOperator::Division => left_value / right_value,
                    BinaryOperator::Exponentiation => left_value.powf(right_value),
                }
            }
            ExpressionNode::UnaryExprNode {
                operator,
                child_node,
            } => {
                let child_value = child_node.evaluate(&vars);
                match operator {
                    UnaryOperator::Sin => child_value.sin(),
                    UnaryOperator::Cos => child_value.cos(),
                    UnaryOperator::Negation => -child_value,
                }
            }
            ExpressionNode::VariableExprNode { variable_index } => vars[*variable_index],
            ExpressionNode::ConstantExprNode { value } => *value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_epression_evaluates_correctly() {
        let complex_expression = ExpressionNode::BinaryExprNode {
            operator: BinaryOperator::Multiplication,
            left_node: Box::new(ExpressionNode::ConstantExprNode { value: 4.0 }),
            right_node: Box::new(ExpressionNode::BinaryExprNode {
                operator: BinaryOperator::Addition,
                left_node: Box::new(ExpressionNode::UnaryExprNode {
                    operator: UnaryOperator::Sin,
                    child_node: Box::new(ExpressionNode::VariableExprNode { variable_index: 0 }),
                }),
                right_node: Box::new(ExpressionNode::ConstantExprNode { value: 3.0 }),
            }),
        };

        assert_eq!(complex_expression.evaluate(&[0.0]), 12.0);
    }
}
