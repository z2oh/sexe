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
                let child_values: Vec<f64> = child_nodes
                                                .iter()
                                                .map(|node| node.evaluate(&vars))
                                                .collect::<Result<_,_>>()?;
                match operator {
                    NaryOperator::Log => if let [a, b] = &child_values[..] {
                            Ok(a.log(*b))
                        }
                        else {
                            Err(EvaluationError::WrongNumberOfArgsError)
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

pub fn evaluate_function_over_domain(
    start_x: f64,
    end_x: f64,
    resolution: u32,
    func: &ExpressionNode,
) -> Vec<(f64, f64)> {
    let mut vars_map = HashMap::new();
    vars_map.insert("x".to_string(), start_x);

    let step_width = (end_x - start_x) / resolution as f64;

    (0..resolution)
        .map(|x| start_x + (x as f64 * step_width))
        .filter_map(|x| {
            if let Some(val) = vars_map.get_mut(&"x".to_string()) {
                *val = x;
            }
            match func.evaluate(&vars_map) {
                Ok(y) => Some((x, y)),
                // For now we simply omit any points that evaluated to an error.
                Err(_) => None,
            }
        })
        .collect()
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
