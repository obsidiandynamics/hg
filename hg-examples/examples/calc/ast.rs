use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self) -> f64;
}

#[derive(Debug, PartialEq)]
pub enum Expression {
    Add(Add),
    Sub(Sub),
    Mult(Mult),
    Div(Div),
    Number(Number)
}

impl Eval for Expression {
    #[inline]
    fn eval(&self) -> f64 {
        match self {
            Expression::Add(eval) => eval.eval(),
            Expression::Sub(eval) => eval.eval(),
            Expression::Mult(eval) => eval.eval(),
            Expression::Div(eval) => eval.eval(),
            Expression::Number(eval) => eval.eval()
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

impl From<Div> for Expression {
    fn from(value: Div) -> Self {
        Expression::Div(value)
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
    #[inline]
    fn eval(&self) -> f64 {
        &self.0.eval() + &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Sub(pub Box<Expression>, pub Box<Expression>);

impl Eval for Sub {
    #[inline]
    fn eval(&self) -> f64 {
        &self.0.eval() - &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Mult(pub Box<Expression>, pub Box<Expression>);

impl Eval for Mult {
    #[inline]
    fn eval(&self) -> f64 {
        &self.0.eval() * &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub struct Div(pub Box<Expression>, pub Box<Expression>);

impl Eval for Div {
    #[inline]
    fn eval(&self) -> f64 {
        &self.0.eval() / &self.1.eval()
    }
}

#[derive(Debug, PartialEq)]
pub enum Number {
    Integer(i64),
    Float(f64)
}

impl Eval for Number {
    #[inline]
    fn eval(&self) -> f64 {
        match *self {
            Number::Integer(i) => i as f64,
            Number::Float(f) => f
        }
    }
}