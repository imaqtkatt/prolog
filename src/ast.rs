use std::fmt;

use crate::mgu::UnificationError;

#[derive(Clone, Debug)]
pub enum Term {
  Var { name: String },
  Num { val: u32 },
  Ctr { name: String, args: Vec<Term> },
}

pub struct Atom(pub String, pub Vec<Term>);

pub struct Head(pub Atom);

pub struct Body(pub Vec<Atom>);

pub enum Clause {
  Fact(Head),
  Rule(Head, Body),
}

pub struct Program(pub Vec<Clause>);

pub struct Goal(pub Vec<Atom>);

pub struct Substitution(pub im_rc::HashMap<String, Term>);

impl Substitution {
  pub fn new(hm: im_rc::HashMap<String, Term>) -> Self {
    Self(hm)
  }

  pub fn empty() -> Self {
    Self(im_rc::HashMap::new())
  }
}

impl fmt::Debug for Substitution {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{:?}", self.0)
  }
}

impl FromIterator<(String, Term)> for Substitution {
  fn from_iter<T: IntoIterator<Item = (String, Term)>>(iter: T) -> Self {
    Self(iter.into_iter().collect())
  }
}

impl Atom {
  pub fn as_ctr(&self) -> Term {
    Term::Ctr {
      name: self.0.clone(),
      args: self.1.clone(),
    }
  }
}

// pub fn compose(s1: Substitution, s2: Substitution) -> Substitution {
//   let mut result = s1
//     .0
//     .into_iter()
//     .map(|(key, value)| (key, value.subst(&s2)))
//     .collect::<Substitution>();

//   result.0.extend(s2.0.into_iter());
//   result
// }

impl Substitution {
  pub fn compose(&self, other: Substitution) -> Self {
    let mut ret = self
      .0
      .clone()
      .into_iter()
      .map(|(k, v)| (k, v.subst(&other)))
      .collect::<Substitution>();
    ret.0.extend(other.0);
    ret
  }
}

pub fn combine<T>(l1: Vec<T>, l2: Vec<T>) -> Vec<(T, T)> {
  l1.into_iter().zip(l2).collect::<Vec<_>>()
}

pub trait Vars {
  fn vars(&self) -> im_rc::Vector<String>;
}

fn union<T: PartialEq + Clone>(
  l1: im_rc::Vector<T>,
  l2: im_rc::Vector<T>,
) -> im_rc::Vector<T> {
  let mut ret = l1.clone();
  for x in l2 {
    if !ret.contains(&x) {
      ret.push_back(x)
    }
  }
  ret
}

impl Vars for Term {
  fn vars(&self) -> im_rc::Vector<String> {
    match self {
      Term::Num { .. } => im_rc::vector![],
      Term::Var { name } => im_rc::vector![name.clone()],
      Term::Ctr { name: _, args } => args
        .iter()
        .fold(im_rc::vector![], |acc, next| union(acc, next.vars())),
    }
  }
}

impl Vars for Atom {
  fn vars(&self) -> im_rc::Vector<String> {
    self.as_ctr().vars()
  }
}

impl Vars for Goal {
  fn vars(&self) -> im_rc::Vector<String> {
    self
      .0
      .iter()
      .fold(im_rc::vector![], |acc, next| union(acc, next.vars()))
  }
}

pub trait Subst {
  fn subst(&self, subs: &Substitution) -> Self;
}

impl Subst for Term {
  fn subst(&self, subs: &Substitution) -> Self {
    match &self {
      Term::Num { .. } => self.clone(),
      Term::Var { name } => {
        if let Some(x) = subs.0.get(name).cloned() {
          x
        } else {
          self.clone()
        }
      }
      Term::Ctr { name, args } => {
        let args = args.iter().map(|arg| arg.subst(subs)).collect::<Vec<_>>();

        Term::Ctr {
          name: name.clone(),
          args,
        }
      }
    }
  }
}

impl Subst for Atom {
  fn subst(&self, subs: &Substitution) -> Self {
    let Self(symbol, args) = self;
    Self(
      symbol.clone(),
      args.iter().map(|arg| arg.subst(subs)).collect::<Vec<_>>(),
    )
  }
}

impl Term {
  pub fn has_var(&self, var: &String) -> bool {
    match self {
      Term::Var { name } => name == var,
      Term::Ctr { name: _, args } => {
        args.iter().fold(false, |acc, arg| acc || arg.has_var(var))
      }
      Term::Num { .. } => false,
    }
  }
}

impl Term {
  pub fn simplify(self) -> Result<Self, UnificationError> {
    use Term::*;
    use UnificationError::*;

    match self {
      Term::Ctr { name, args } if is_op(&name) => {
        match (name.as_str(), &args[..]) {
          ("+", [x, y]) => match (x.clone().simplify()?, y.clone().simplify()?)
          {
            (Num { val: x }, Num { val: y }) => Ok(Num { val: x + y }),
            _ => Err(NotUnifiable),
          },
          ("-", [x, y]) => match (x.clone().simplify()?, y.clone().simplify()?)
          {
            (Num { val: x }, Num { val: y }) => Ok(Num { val: x - y }),
            _ => Err(NotUnifiable),
          },
          ("*", [x, y]) => match (x.clone().simplify()?, y.clone().simplify()?)
          {
            (Num { val: x }, Num { val: y }) => Ok(Num { val: x * y }),
            _ => Err(NotUnifiable),
          },
          ("/", [x, y]) => match (x.clone().simplify()?, y.clone().simplify()?)
          {
            (Num { val: x }, Num { val: y }) => Ok(Num { val: x / y }),
            _ => Err(NotUnifiable),
          },
          _ => Err(NotUnifiable),
        }
      }
      _ => Ok(self),
    }
  }
}

fn is_op(name: &str) -> bool {
  match name {
    "+" | "-" | "*" | "/" => true,
    _ => false,
  }
}
