/// All expression nodes implement the ExprNode trait, which takes an array of floats that contain
/// the values of variables. For example, in the expression `2x + 3y`, `x` has the variable index
/// of `0` and `y` has the variable index of `1`. To evaluate this expression with `x` equal to 3
/// and `y` equal to -2, our `vars` variable would look like this: [3.0, -2.0].
pub trait ExprNode {
    fn evaluate(&self, vars: &[f64]) -> f64;
}

/// These are the supported binary operators.
/// Addition: `+`
/// Subtraction: `-`
/// Multiplication: `*`
/// Division: `/`
/// Exponentiation: '^'
#[allow(dead_code)]
pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Exponentiation,
}

/// These are the supported unary operators.
/// Sin: `sin()`
/// Cos: `cos()`
/// Negation: `-`, as in `-4`
#[allow(dead_code)]
pub enum UnaryOperator {
    Sin,
    Cos,
    Negation,
}

/// The binary expression node has a binary operator and a left and right node.
#[allow(dead_code)]
pub struct BinaryExprNode<T: ExprNode, U: ExprNode> {
    pub operator: BinaryOperator,
    pub left_node: T,
    pub right_node: U,
}

/// The unary expression node has a unary operator and a child node.
#[allow(dead_code)]
pub struct UnaryExprNode<T: ExprNode> {
    pub operator: UnaryOperator,
    pub child_node: T,
}

/// The constant expression node is just a wrapper around a constant value.
#[allow(dead_code)]
pub struct ConstantExprNode {
    pub value: f64,
}

/// The variable expression node has an index representing the variable that the node represents.
/// This value is used to index into the `vars` argument passed to the `evaluate` function.
#[allow(dead_code)]
pub struct VariableExprNode {
    pub variable_index: usize,
}

impl<T: ExprNode, U: ExprNode> ExprNode for BinaryExprNode<T, U> {
    fn evaluate(&self, vars: &[f64]) -> f64 {
        let left_value = self.left_node.evaluate(vars);
        let right_value = self.right_node.evaluate(vars);
        match self.operator {
            BinaryOperator::Addition => left_value + right_value,
            BinaryOperator::Subtraction => left_value - right_value,
            BinaryOperator::Multiplication => left_value * right_value,
            BinaryOperator::Division => left_value / right_value,
            BinaryOperator::Exponentiation => left_value.powf(right_value),
        }
    }
}

impl<T: ExprNode> ExprNode for UnaryExprNode<T> {
    fn evaluate(&self, vars: &[f64]) -> f64 {
        let child_value = self.child_node.evaluate(vars);
        match self.operator {
            UnaryOperator::Sin => child_value.sin(),
            UnaryOperator::Cos => child_value.cos(),
            UnaryOperator::Negation => -child_value,
        }
    }
}

impl ExprNode for ConstantExprNode {
    fn evaluate(&self, _vars: &[f64]) -> f64 {
        self.value
    }
}

impl ExprNode for VariableExprNode {
    fn evaluate(&self, vars: &[f64]) -> f64 {
        vars[self.variable_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_epression_evaluates_correctly() {
        // This expression represents the equation
        // 4*(sin(x) + 3)
        let expr = BinaryExprNode {
            operator: BinaryOperator::Multiplication,
            left_node: ConstantExprNode { value: 4.0 },
            right_node: BinaryExprNode {
                operator: BinaryOperator::Addition,
                left_node: UnaryExprNode {
                    operator: UnaryOperator::Sin,
                    child_node: VariableExprNode { variable_index: 0 },
                },
                right_node: ConstantExprNode { value: 3.0 },
            },
        };
        assert_eq!(expr.evaluate(&[0.0]), 12.0);
    }
}
