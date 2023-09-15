use crate::types::{Hole, HoleKind, Type, TypeKind};

/// Verifies if we are unifying with an infite Type.
pub fn occurs_check(hole: Hole, t: Type) -> bool {
  match &*t {
    TypeKind::Var(_) => false,
    TypeKind::Hole(inner) => inner.clone() == hole,
    TypeKind::Arrow(t1, t2) => {
      occurs_check(hole.clone(), t1.clone()) || occurs_check(hole, t2.clone())
    }
  }
}

pub fn unify(t1: Type, t2: Type) -> bool {
  use TypeKind::*;

  match (&*t1, &*t2) {
    (Var(left), Var(right)) if left == right => true,

    (Hole(left), Hole(right)) if left == right => true,

    (Hole(inner), _) => unify_hole(inner.clone(), t2.clone(), false),
    (_, Hole(inner)) => unify_hole(inner.clone(), t1.clone(), true),

    (Arrow(l1, r1), Arrow(l2, r2)) => {
      let left_unifies = unify(l1.clone(), l2.clone());
      let right_unifies = unify(r1.clone(), r2.clone());
      left_unifies && right_unifies
    }
    _ => false,
  }
}

pub fn unify_hole(hole: Hole, t: Type, flip: bool) -> bool {
  match hole.get() {
    HoleKind::Empty(_, _) => {
      if occurs_check(hole.clone(), t.clone()) {
        panic!("Occurs check")
      } else {
        hole.fill_with(t);
        true
      }
    }
    // HoleKind::Filled(filled) if flip => unify(filled, t),
    HoleKind::Filled(filled) => {
      if flip {
        unify(t, filled)
      } else {
        unify(filled, t)
      }
    }
  }
}
