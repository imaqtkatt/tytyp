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
