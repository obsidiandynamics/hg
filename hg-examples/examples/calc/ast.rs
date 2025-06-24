use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self) -> f64;
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Add(Add),
    Sub(Sub),
    Mult(Mult),
    Number(Number)
}

impl Eval for Expression {
    fn eval(&self) -> f64 {
        match self {
            Expression::Add(add) => add.eval(),
            Expression::Sub(sub) => sub.eval(),
            Expression::Mult(mult) => mult.eval(),
            Expression::Number(number) => number.eval()
        }
    }
}

impl From<Add> for Expression {
    fn from(value: Add) -> Self {
        Expression::Add(value)
    }
}

impl From<Sub> for Expression {
    fn from(value: Sub) -> Self {
        Expression::Sub(value)
    }
}

impl From<Mult> for Expression {
    fn from(value: Mult) -> Self {
        Expression::Mult(value)
    }
}

impl From<Number> for Expression {
    fn from(value: Number) -> Self {
        Expression::Number(value)
    }
}

#[derive(Debug, PartialEq)]
pub struct Add(pub Box<Expression>, pub Box<Expression>);

impl Eval for Add {
    fn eval(&self) -> f64 {
        &self.0.eval() + &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Sub(pub Box<Expression>, pub Box<Expression>);

impl Eval for Sub {
    fn eval(&self) -> f64 {
        &self.0.eval() - &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Mult(pub Box<Expression>, pub Box<Expression>);

impl Eval for Mult {
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