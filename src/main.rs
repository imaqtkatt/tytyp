#![allow(unused_variables)]

use expr::ExprKind;
use infer::Context;

pub mod expr;
pub mod infer;
pub mod types;
pub mod unification;

use ExprKind::*;

use crate::expr::{Annot, Expr, Lit::*};

fn main() -> Result<(), std::string::String> {
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

  let apply: Expr = Lam {
    var: "f".into(),
    body: Lam {
      var: "arg".into(),
      body: App {
        fun: Var { name: "f".into() }.into(),
        arg: Var { name: "arg".into() }.into(),
      }
      .into(),
    }
    .into(),
  }
  .into();

  // (fst 1)
  let fst_applied_to_one: Expr = App {
    fun: fst.clone(),
    arg: Lit { val: Int(1) }.into(),
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
    fun: identity.clone(),
    arg: Lit {
      val: String("hey!".into()),
    }
    .into(),
  }
  .into();

  let id_bool: Expr = LamTyp {
    annot: Annot {
      var: "b".into(),
      t: types::bool(),
    },
    body: Var { name: "b".into() }.into(),
  }
  .into();

  let app_id_bool_to_string: Expr = App {
    fun: id_bool.clone(),
    arg: Lit {
      val: String("why?".into()),
    }
    .into(),
  }
  .into();

  let let_batata_annot_string_in_batata: Expr = LetTyp {
    annot: Annot {
      var: "batata".into(),
      t: types::string(),
    },
    val: Lit {
      val: String("pure de batata".into()),
    }
    .into(),
    next: Var {
      name: "batata".into(),
    }
    .into(),
  }
  .into();

  let let_batata_annot_int_in_batata: Expr = LetTyp {
    annot: Annot {
      var: "batata".into(),
      t: types::string(),
    },
    val: Lit { val: Int(42) }.into(),
    next: Var {
      name: "batata".into(),
    }
    .into(),
  }
  .into();

  let r#let = Let {
    binding: "bar".into(),
    val: Lam {
      var: "x".into(),
      body: Var { name: "x".into() }.into(),
    }
    .into(),
    next: Var { name: "bar".into() }.into(),
  }
  .into();

  let mut ctx = Context::default();

  let (e, t) = ctx.infer(r#let)?;

  println!("{e}\n|-\n{t}");

  Ok(())
}
