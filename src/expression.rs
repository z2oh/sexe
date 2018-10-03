use std::collections::HashMap;

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
    /// Tan: `tan()`
    Tan,
    /// Abs: `abs()`
    Abs,
    /// Exp: `exp()`
    Exp,
    /// Log2: `log2()`
    Log2,
    /// Ln: `ln()`
    Ln,
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
    VariableExprNode { variable_key: String },
    /// This variant holds a constant value.
    ConstantExprNode { value: f64 },
}

#[derive(Debug)]
pub enum EvaluationError {
    VariableNotFoundError,
    IllegalArithmeticError,
}

impl ExpressionNode {
    /// Takes in an array of variables to recursively pass down to all `ExpressionNode`s until the
    /// expression is evaluated. The `f64` value returned is the result of the expression tree
    /// rooted at `self`.
    pub fn evaluate(&self, vars: &HashMap<String, f64>) -> Result<f64, EvaluationError> {
        match self {
            ExpressionNode::BinaryExprNode {
                operator,
                left_node,
                right_node,
            } => {
                let left_value = left_node.evaluate(&vars)?;
                let right_value = right_node.evaluate(&vars)?;
                match operator {
                    BinaryOperator::Addition => Ok(left_value + right_value),
                    BinaryOperator::Subtraction => Ok(left_value - right_value),
                    BinaryOperator::Multiplication => Ok(left_value * right_value),
                    BinaryOperator::Division => Ok(left_value / right_value),
                    BinaryOperator::Exponentiation => Ok(left_value.powf(right_value)),
                }
            }
            ExpressionNode::UnaryExprNode {
                operator,
                child_node,
            } => {
                let child_value = child_node.evaluate(&vars)?;
                match operator {
                    UnaryOperator::Sin => Ok(child_value.sin()),
                    UnaryOperator::Cos => Ok(child_value.cos()),
                    UnaryOperator::Tan => Ok(child_value.tan()),
                    UnaryOperator::Negation => Ok(-child_value),
                    UnaryOperator::Abs => Ok(child_value.abs()),
                    UnaryOperator::Exp => Ok(child_value.exp()),
                    // We must wrap problematic functions to prevent errors
                    UnaryOperator::Log2 => match child_value {
                        x if (x>0.0f64) => Ok(x.log2()),
                        _ => Err(EvaluationError::IllegalArithmeticError)
                    },
                    UnaryOperator::Ln => match child_value {
                        x if (x>0.0f64) => Ok(x.ln()),
                        _ => Err(EvaluationError::IllegalArithmeticError)
                    },
                }
            }
            ExpressionNode::VariableExprNode { variable_key } => match vars.get(variable_key) {
                Some(x) => Ok(*x),
                None => Err(EvaluationError::VariableNotFoundError),
            },
            ExpressionNode::ConstantExprNode { value } => Ok(*value),
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
                    child_node: Box::new(ExpressionNode::VariableExprNode {
                        variable_key: "x".to_string(),
                    }),
                }),
                right_node: Box::new(ExpressionNode::ConstantExprNode { value: 3.0 }),
            }),
        };

        let mut vars_map = HashMap::new();
        vars_map.insert("x".to_string(), 0.0);

        assert_eq!(complex_expression.evaluate(&vars_map).unwrap(), 12.0);
    }
}
