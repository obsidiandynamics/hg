use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self) -> f64;
}

#[derive(Debug, PartialEq)]
pub enum EvalKind {
    Sum(Sum),
    Product(Product),
    Number(Number)
}

impl Eval for EvalKind {
    fn eval(&self) -> f64 {
        match self {
            EvalKind::Sum(sum) => sum.eval(),
            EvalKind::Product(product) => product.eval(),
            EvalKind::Number(number) => number.eval()
        }
    }
}

impl From<Sum> for EvalKind {
    fn from(value: Sum) -> Self {
        EvalKind::Sum(value)
    }
}

impl From<Product> for EvalKind {
    fn from(value: Product) -> Self {
        EvalKind::Product(value)
    }
}

impl From<Number> for EvalKind {
    fn from(value: Number) -> Self {
        EvalKind::Number(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct Sum(pub Box<EvalKind>, pub Box<EvalKind>);

impl Eval for Sum {
    fn eval(&self) -> f64 {
        &self.0.eval() + &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Product(pub Box<EvalKind>, pub Box<EvalKind>);

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