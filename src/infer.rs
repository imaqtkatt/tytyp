use std::{
  cell::{Ref, RefCell},
  collections::HashMap,
};

use crate::{
  expr::{self, Annot, Expr, ExprKind},
  types::{self, Hole, HoleKind, Scheme, Type, TypeKind},
  unification::unify,
};

/// Our context keeps track of all types, the current level, and
/// can generate new [Hole] types.
#[derive(Clone)]
pub struct Context {
  pub current_id: RefCell<u32>,
  pub current_level: RefCell<u32>,
  pub types: HashMap<String, Scheme>,
}

impl Default for Context {
  fn default() -> Self {
    Self {
      current_id: Default::default(),
      current_level: Default::default(),
      types: Default::default(),
    }
  }
}

type InferResult = Result<(Expr, Type), String>;

impl Context {
  /// Returns a new name based on the current id and level.
  pub fn new_hole_type(&self) -> Type {
    let level = *self.current_level.borrow();

    let hole = Hole::new(self.new_name(), level);
    Type::new(TypeKind::Hole(hole))
  }

  fn new_name(&self) -> String {
    let mut id = self.current_id.borrow_mut();
    let name = format!("t_{i}", i = *id).to_string();
    *id += 1;
    name
  }

  pub fn enter_level(&self) {
    *self.current_level.borrow_mut() += 1;
  }

  pub fn exit_level(&self) {
    *self.current_level.borrow_mut() -= 1;
  }

  /// The instantiation transforms a PolyType into a MonoType.
  pub fn instantiate(&self, scheme: Scheme) -> Type {
    let substitutions = scheme
      .binds
      .iter()
      .map(|_| self.new_hole_type())
      .collect::<Vec<_>>();

    scheme.t.instantiate(&substitutions)
  }

  /// The generalization transforms a MonoType into a PolyType.
  pub fn generalize(&self, t: Type) -> Scheme {
    let mut counter = 0;
    let level = self.current_level.borrow();

    fn gen(t: Type, level: &Ref<'_, u32>, counter: &mut u32) {
      match &*t {
        TypeKind::Hole(inner) => match inner.get() {
          HoleKind::Empty(_, hole_level) if hole_level > **level => {
            let curr_level = *counter;
            *counter += 1;
            inner.fill_with(Type::new(TypeKind::Generalized(curr_level)))
          }
          HoleKind::Empty(_, _) => (),
          HoleKind::Filled(t) => gen(t, level, counter),
        },
        TypeKind::Arrow(t1, t2) => {
          gen(t1.clone(), level, counter);
          gen(t2.clone(), level, counter);
        }
        _ => (),
      };
    }

    gen(t.clone(), &level, &mut counter);

    let binds = (0..counter).map(|_| self.new_name()).collect::<Vec<_>>();

    Scheme::new(binds, t)
  }

  pub fn infer(&mut self, expr: Expr) -> InferResult {
    match &*expr {
      // Literals are the easiest ones,
      // we already know their types!
      ExprKind::Lit { val } => {
        let t = match val {
          expr::Lit::Int(_) => types::int(),
          expr::Lit::Bool(_) => types::bool(),
          expr::Lit::String(_) => types::string(),
        };
        Ok((expr, t))
      }
      // If it is a Var, we need to check the existence of the variable
      // in our context and instantiate it.
      //
      //  if name : S existing in Γ
      // ---------------------------
      //        name : inst(S)
      ExprKind::Var { name } => {
        let scheme = match self.types.get(name).cloned() {
          Some(t) => t,
          None => panic!("Cannot infer '{name}'"),
        };
        let instantiated = self.instantiate(scheme);
        Ok((expr, instantiated))
      }
      // If it is a Lambda, we need to generate a new Scheme and extend
      // [var => scheme] to a new context and infer the body.
      //
      //  t = hole.  extend Γ, var : t, infer(body) : body_t
      // ----------------------------------------------------
      //               (λvar. body) : t -> body_t
      ExprKind::Lam { var, body } => {
        let hole = self.new_hole_type();
        let scheme = Scheme::new(vec![], hole.clone());

        let mut new_env = self.clone();
        new_env.types.insert(var.clone(), scheme);

        let (_, body_t) = new_env.infer(body.clone())?;

        let fun_t = TypeKind::Arrow(hole, body_t);
        Ok((expr, Type::new(fun_t)))
      }
      // if it is a Typed Lambda, we already know the type of the var.
      //
      //  var : t.  extend Γ, var : gen(t), infer(body) : body_t
      // --------------------------------------------------------
      //              (λvar. body) : t -> body_t
      // ExprKind::LamTyp { var, t, body } => {
      ExprKind::LamTyp {
        annot: Annot { var, t },
        body,
      } => {
        let mut new_env = self.clone();
        let generalized = new_env.generalize(t.clone());

        new_env.types.insert(var.clone(), generalized.clone());

        let (_, body_t) = new_env.infer(body.clone())?;

        let fun_t = TypeKind::Arrow(t.clone(), body_t);
        Ok((expr, Type::new(fun_t)))
      }
      // If it is an App, we need to infer both function and argument.
      // The function type needs to be `t -> t'`, because of that, we
      // need to unify it with a new arrow type from `arg_t -> hole`.
      //
      //  infer(fun) : fun_t, infer(arg_t) : arg_t, hole = t'
      // -----------------------------------------------------
      //              unify(fun_t, arg_t -> hole)
      //             -----------------------------
      //                     (fun arg) : t'
      ExprKind::App { fun, arg } => {
        let (_, fun_t) = self.infer(fun.clone())?;
        let (_, arg_t) = self.infer(arg.clone())?;

        let hole = self.new_hole_type();

        let arrow_t = Type::new(TypeKind::Arrow(arg_t, hole.clone()));

        unify(fun_t, arrow_t.clone())?;
        Ok((expr, hole))
      }
      // If it is a Let expr, we need to increment our level, generalize the
      // inferred value and extend it to a new context to infer the next expr.
      //
      //  infer(val) : val_t. extend Γ, binding : gen(val_t), infer(next) : t'
      // ----------------------------------------------------------------------
      //                  (let binding = val in next) : t'
      ExprKind::Let { binding, val, next } => {
        self.enter_level();
        let (_, val_t) = self.infer(val.clone())?;
        self.exit_level();

        let val_generalized = self.generalize(val_t);

        let mut new_env = self.clone();
        new_env.types.insert(binding.clone(), val_generalized);

        let (_, next_t) = new_env.infer(next.clone())?;

        Ok((expr, next_t))
      }
      ExprKind::LetTyp {
        annot: Annot { var, t },
        val,
        next,
      } => {
        let (_, val_t) = self.infer(val.clone())?;
        unify(val_t, t.clone())?;

        let t_annot_gen = self.generalize(t.clone());

        let mut new_env = self.clone();
        new_env.types.insert(var.clone(), t_annot_gen);

        let (_, next_t) = new_env.infer(next.clone())?;

        Ok((expr, next_t))
      }
    }
  }
}
