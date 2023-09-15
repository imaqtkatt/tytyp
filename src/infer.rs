use std::{cell::RefCell, collections::HashMap};

use crate::{
  expr::{self, Expr, ExprKind},
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

  /// Returns a free variable that is not defined inside an expression.
  pub fn free_vars(&self, t: Type) -> Vec<String> {
    match &*t {
      TypeKind::Var(_) => vec![],
      TypeKind::Arrow(t1, t2) => {
        let mut t1_free_vars = self.free_vars(t1.clone());
        let mut t2_free_vars = self.free_vars(t2.clone());
        t1_free_vars.append(&mut t2_free_vars);

        t1_free_vars
      }
      TypeKind::Hole(hole) => match &*hole.0.borrow() {
        HoleKind::Filled(t) => self.free_vars(t.clone()),
        HoleKind::Empty(name, level) => {
          if *level > *self.current_level.borrow() {
            vec![name.clone()]
          } else {
            vec![]
          }
        }
      },
    }
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
    let mut free_vars = self.free_vars(t.clone());
    free_vars.dedup();
    free_vars.sort();

    Scheme::new(free_vars, t)
  }

  pub fn infer(&mut self, expr: Expr) -> (Expr, Type) {
    match &*expr {
      // Literals are the easiest ones,
      // we already know their types!
      ExprKind::Lit { val } => {
        let t = match val {
          expr::Lit::Int(_) => types::int(),
          expr::Lit::Bool(_) => types::bool(),
          expr::Lit::String(_) => types::string(),
        };
        (expr, t)
      }
      // If it is a Var, we need to check the existence of the variable
      // in our context and instantiate it.
      ExprKind::Var { name } => {
        let scheme = match self.types.get(name).cloned() {
          Some(t) => t,
          None => panic!("Cannot infer '{name}'"),
        };
        let instantiated = self.instantiate(scheme);
        (expr, instantiated)
      }
      // If it is a Lambda, we need to generate a new Scheme and add the
      // [var => scheme] to a new context and infer the body.
      ExprKind::Lam { var, body } => {
        let hole = self.new_hole_type();
        let scheme = Scheme::new(vec![], hole.clone());

        let mut new_env = self.clone();
        new_env.types.insert(var.clone(), scheme);

        let (_, body_t) = new_env.infer(body.clone());

        let fun_t = TypeKind::Arrow(hole, body_t);
        (expr, Type::new(fun_t))
      }
      ExprKind::App { fun, arg } => {
        let (_, fun_t) = self.infer(fun.clone());
        let (_, arg_t) = self.infer(arg.clone());

        let hole = self.new_hole_type();

        let arrow_t = Type::new(TypeKind::Arrow(arg_t, hole.clone()));

        unify(fun_t, arrow_t.clone());
        (expr, hole)
      }
      // If it is a Let expr, we need to increment our level and generalize the
      // inferred value and add it to a new context to infer the next expr.
      ExprKind::Let { binding, val, next } => {
        self.enter_level();
        let (_, val_t) = self.infer(val.clone());
        self.exit_level();

        let val_generalized = self.generalize(val_t);

        let mut new_env = self.clone();
        new_env.types.insert(binding.clone(), val_generalized);

        let (_, next_t) = new_env.infer(next.clone());

        (expr, next_t)
      }
    }
  }
}
