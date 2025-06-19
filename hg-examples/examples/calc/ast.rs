use std::fmt::Debug;

pub trait Eval: Debug {
    fn eval(&self) -> f64;
}

pub type DynEval = Box<dyn Eval>;

#[derive(Debug)]
pub struct Sum(pub DynEval, pub DynEval);

impl Eval for Sum {
    fn eval(&self) -> f64 {
        &self.0.eval() + &self.1.eval()
    }
}

#[derive(Debug)]
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