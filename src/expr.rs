use std::fmt::{Debug, Display};

#[derive(Clone, Debug)]
pub enum Lit {
  Int(i32),
  Bool(bool),
  String(String),
}

#[derive(Clone, Debug)]
pub enum ExprKind {
  Var {
    name: String,
  },
  Lit {
    val: Lit,
  },
  Lam {
    var: String,
    body: Expr,
  },
  App {
    fun: Expr,
    arg: Expr,
  },
  Let {
    binding: String,
    val: Expr,
    next: Expr,
  },
}

pub type Expr = Box<ExprKind>;

impl Display for Lit {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Lit::Int(i) => write!(f, "{i}"),
      Lit::Bool(b) => write!(f, "{b}"),
      Lit::String(s) => write!(f, "{s}"),
    }
  }
}

impl Display for ExprKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Var { name } => write!(f, "{name}"),
      Self::Lit { val } => write!(f, "{val}"),
      Self::Lam { var, body } => write!(f, "λ{var}. {body}"),
      Self::App { fun, arg } => write!(f, "({fun} {arg})"),
      Self::Let { binding, val, next } => {
        write!(f, "let {binding} = {val} in\n  {next}")
      }
    }
  }
}
