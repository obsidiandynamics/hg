use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self) -> f64;
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Sum(Sum),
    Product(Product),
    Number(Number)
}

impl Eval for Expression {
    fn eval(&self) -> f64 {
        match self {
            Expression::Sum(sum) => sum.eval(),
            Expression::Product(product) => product.eval(),
            Expression::Number(number) => number.eval()
        }
    }
}

impl From<Sum> for Expression {
    fn from(value: Sum) -> Self {
        Expression::Sum(value)
    }
}

impl From<Product> for Expression {
    fn from(value: Product) -> Self {
        Expression::Product(value)
    }
}

impl From<Number> for Expression {
    fn from(value: Number) -> Self {
        Expression::Number(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct Sum(pub Box<Expression>, pub Box<Expression>);

impl Eval for Sum {
    fn eval(&self) -> f64 {
        &self.0.eval() + &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Product(pub Box<Expression>, pub Box<Expression>);

impl Eval for Product {
    fn eval(&self) -> f64 {
        &self.0.eval() * &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64)
}

impl Eval for Number {
    fn eval(&self) -> f64 {
        match *self {
            Number::Integer(i) => i as f64,
            Number::Float(f) => f
        }
    }
}