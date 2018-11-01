use std::collections::HashMap;

/// These are the supported binary operators.
#[derive(Debug, PartialEq)]
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
#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
    /// Sin: `sin()`
    Sin,
    /// Cos: `cos()`
    Cos,
    /// Tan: `tan()`
    Tan,
    /// Ctan `1.0 / tan()`
    Ctan,
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
    /// Asin: `asin()`
    Asin,
    /// Acos: `acos()`
    Acos,
}

/// These are the supported N-ary operators.
#[derive(Debug, PartialEq)]
pub enum NaryOperator {
    /// Log: `log(base, x)`
    Log,
}

/// An expression node is any part of the parsed expression tree. These build up the expression
/// recursively. Every value, variable, and operator is wrapped in an `ExpressionNode`.
#[derive(Debug, PartialEq)]
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
    /// This variant holds an operator that is to be applied to evaluated values of its child subtrees.
    NaryExprNode {
        operator: NaryOperator,
        child_nodes: Box<Vec<ExpressionNode>>,
    },
    /// This variant holds an index into the `vars` array which indicates which variable of the
    /// expression it represents.
    VariableExprNode { variable_key: String },
    /// This variant holds a constant value.
    ConstantExprNode { value: f64 },
}

#[derive(Debug, PartialEq)]
pub enum EvaluationError {
    VariableNotFoundError,
    WrongNumberOfArgsError,
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
                    UnaryOperator::Ctan => Ok(1.0 / child_value.tan()),
                    UnaryOperator::Negation => Ok(-child_value),
                    UnaryOperator::Abs => Ok(child_value.abs()),
                    UnaryOperator::Exp => Ok(child_value.exp()),
                    UnaryOperator::Log2 => Ok(child_value.log2()),
                    UnaryOperator::Ln => Ok(child_value.ln()),
                    UnaryOperator::Asin => Ok(child_value.asin()),
                    UnaryOperator::Acos => Ok(child_value.acos()),
                }
            }
            ExpressionNode::NaryExprNode {
                operator,
                child_nodes,
            } => {
                let mut child_values: Vec<f64> = Vec::new();
                for node in child_nodes.iter() {
                    child_values.push(node.evaluate(&vars)?);
                }
                match operator {
                    NaryOperator::Log => match child_values.len() {
                        2 => Ok(child_values[0].log(child_values[1])),
                        _ => Err(EvaluationError::WrongNumberOfArgsError),
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
    fn complex_expression_evaluates_correctly() {
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
