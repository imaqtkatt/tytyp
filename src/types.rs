use std::{cell::RefCell, fmt::Display, rc::Rc};

/// Represents a MonoType.
#[derive(Clone, Debug)]
pub enum TypeKind {
  Var(String),
  Hole(Hole),
  Generalized(u32),
  Arrow(Type, Type),
}

// As a TypeKind is recursive and have multiple sizes,
// we need to point it to the heap memory.
pub type Type = Box<TypeKind>;

// Literal types

pub fn string() -> Type {
  Type::new(TypeKind::Var("String".into()))
}

pub fn int() -> Type {
  Type::new(TypeKind::Var("Int".into()))
}

pub fn bool() -> Type {
  Type::new(TypeKind::Var("Bool".into()))
}

/// Represents the inner content of a [Hole].
#[derive(Clone, Debug)]
pub enum HoleKind {
  Filled(Type),
  Empty(String, u32),
}

/// A Hole represents a type that was not discovered yet.
#[derive(Clone, Debug)]
pub struct Hole(pub Rc<RefCell<HoleKind>>);

impl Hole {
  /// Fills an empty Hole with the given [Type].
  pub fn fill_with(&self, t: Type) {
    *self.0.borrow_mut() = HoleKind::Filled(t)
  }

  pub fn new(name: String, level: u32) -> Self {
    let hole_kind = HoleKind::Empty(name, level);
    let inner = Rc::new(RefCell::new(hole_kind));
    Self(inner)
  }

  pub fn get_mut(&self) -> std::cell::RefMut<'_, HoleKind> {
    self.0.borrow_mut()
  }

  pub fn get(&self) -> HoleKind {
    self.0.borrow().clone()
  }
}

impl PartialEq for Hole {
  fn eq(&self, other: &Self) -> bool {
    Rc::ptr_eq(&self.0, &other.0)
  }
}

impl Eq for Hole {}

/// Represents a Polymorphic Type, like what happens in the identity function.
///
/// ```
/// id : forall a. a -> a
/// ```
#[derive(Clone)]
pub struct Scheme {
  pub binds: Vec<String>,
  pub t: Type,
}

impl Scheme {
  pub fn new(binds: Vec<String>, t: Type) -> Self {
    Self { binds, t }
  }
}

impl TypeKind {
  pub fn instantiate(self: Type, substitutions: &[Type]) -> Type {
    match &&*self {
      TypeKind::Var(_) => self.clone(),
      TypeKind::Generalized(n) => substitutions[*n as usize].clone(),
      TypeKind::Arrow(t1, t2) => {
        let t1 = t1.clone().instantiate(substitutions);
        let t2 = t2.clone().instantiate(substitutions);

        Type::new(TypeKind::Arrow(t1, t2))
      }
      TypeKind::Hole(hole) => match hole.get() {
        HoleKind::Filled(t) => t.clone().instantiate(substitutions),
        HoleKind::Empty(_, _) => self.clone(),
      },
    }
  }
}

impl Display for TypeKind {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      TypeKind::Var(name) => write!(f, "{name}"),
      TypeKind::Hole(inner) => match inner.get() {
        HoleKind::Filled(t) => write!(f, "{t}"),
        HoleKind::Empty(id, _) => write!(f, "{id}"),
      },
      TypeKind::Generalized(n) => write!(f, "gen_{n}"),
      TypeKind::Arrow(t1, t2) => {
        if need_parens(t1.clone()) {
          write!(f, "({t1}) -> {t2}")
        } else {
          write!(f, "{t1} -> {t2}")
        }
      }
    }
  }
}

#[inline(always)]
fn need_parens(t: Type) -> bool {
  match *t {
    TypeKind::Arrow(_, _) => true,
    TypeKind::Hole(inner) => match inner.get() {
      HoleKind::Filled(t) => need_parens(t),
      HoleKind::Empty(_, _) => false,
    },
    _ => false,
  }
}
