#![allow(unused_variables)]

use expr::ExprKind;
use infer::Context;

pub mod expr;
pub mod infer;
pub mod types;
pub mod unification;

use ExprKind::*;

use crate::expr::{Expr, Lit::String};

fn main() {
  // id = λx. x
  let identity: Expr = Lam {
    var: "x".into(),
    body: Var { name: "x".into() }.into(),
  }
  .into();

  // fst = λa.λb. a
  let fst: Expr = Lam {
    var: "a".into(),
    body: Lam {
      var: "b".into(),
      body: Var { name: "a".into() }.into(),
    }
    .into(),
  }
  .into();

  // let id = λx. x in (id id)
  let let_id_in_id_id: Expr = Let {
    binding: "id".into(),
    val: identity.clone(),
    next: App {
      fun: identity.clone(),
      arg: identity.clone(),
    }
    .into(),
  }
  .into();

  // (id "hey!")
  let app_identity_string: Expr = App {
    fun: identity,
    arg: Lit {
      val: String("hey!".into()),
    }
    .into(),
  }
  .into();

  let mut ctx = Context::default();

  let (e, t) = ctx.infer(let_id_in_id_id);
  let t = t.force();

  println!("{e:?}\n|-\n{t:?}");
}
