use std::fmt::{Debug, Display};

use crate::types::Type;

/// Lit is a literal type.
#[derive(Clone, Debug)]
pub enum Lit {
  Int(i32),
  Bool(bool),
  String(String),
}

#[derive(Clone, Debug)]
pub struct Annot {
  pub var: String,
  pub t: Type,
}

/// Represents the tree we want to type check.
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
  LamTyp {
    annot: Annot,
    body: Expr,
  },
  LetTyp {
    annot: Annot,
    val: Expr,
    next: Expr,
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

// Same thing occurs in Type, we need to point it
// to the heap memory.
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
      Self::LamTyp {
        annot: Annot { var, t },
        body,
      } => {
        write!(f, "λ{var} : {t}. {body}")
      }
      Self::LetTyp {
        annot: Annot { var, t },
        val,
        next,
      } => {
        write!(f, "let ({var} : {t}) = {val} in\n  {next}")
      }
      Self::App { fun, arg } => write!(f, "({fun} {arg})"),
      Self::Let { binding, val, next } => {
        write!(f, "let {binding} = {val} in\n  {next}")
      }
    }
  }
}
