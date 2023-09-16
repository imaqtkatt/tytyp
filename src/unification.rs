use crate::types::{Hole, HoleKind, Type, TypeKind};

/// Verifies if we are unifying with an infite Type.
pub fn occurs_check(hole: Hole, t: Type) -> bool {
  match &*t {
    TypeKind::Var(_) | TypeKind::Generalized(_) => false,
    TypeKind::Hole(inner) => inner.clone() == hole,
    TypeKind::Arrow(t1, t2) => {
      occurs_check(hole.clone(), t1.clone()) || occurs_check(hole, t2.clone())
    }
  }
}

type UnifyResult = Result<(), String>;

pub fn unify(t1: Type, t2: Type) -> UnifyResult {
  use TypeKind::*;

  match (&*t1, &*t2) {
    (Var(left), Var(right)) if left == right => Ok(()),

    (Generalized(left), Generalized(right)) if left == right => Ok(()),

    (Hole(left), Hole(right)) if left == right => Ok(()),
    (Hole(inner), _) => unify_hole(inner.clone(), t2.clone(), false),
    (_, Hole(inner)) => unify_hole(inner.clone(), t1.clone(), true),

    (Arrow(l1, r1), Arrow(l2, r2)) => {
      unify(l1.clone(), l2.clone())?;
      unify(r1.clone(), r2.clone())
    }
    _ => Err(format!("Cannot unify {t1} with {t2}")),
  }
}

pub fn unify_hole(hole: Hole, t: Type, flip: bool) -> UnifyResult {
  match hole.get() {
    HoleKind::Empty(_, _) => {
      if occurs_check(hole.clone(), t.clone()) {
        panic!("Occurs check")
      } else {
        hole.fill_with(t);
        Ok(())
      }
    }
    HoleKind::Filled(filled) => {
      if flip {
        unify(t, filled)
      } else {
        unify(filled, t)
      }
    }
  }
}
