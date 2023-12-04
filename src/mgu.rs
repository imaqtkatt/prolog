use crate::ast::{combine, Atom, Subst, Substitution as Subs, Term};

trait Mgu {
  type ErrType;

  fn mgu(&self, other: &Self) -> Result<Subs, Self::ErrType>;
}

#[derive(Debug)]
pub enum UnificationError {
  NotUnifiable,
}

impl Mgu for Term {
  type ErrType = UnificationError;

  fn mgu(&self, other: &Self) -> Result<Subs, Self::ErrType> {
    use im_rc::hashmap;
    use Term::*;
    use UnificationError::*;

    match (self, other) {
      (Var { name: x }, var @ Var { name: y }) => {
        if x == y {
          Ok(Subs::empty())
        } else {
          Ok(Subs::new(hashmap! { x.clone() => var.clone() }))
        }
      }
      (Var { name }, ctr @ Ctr { .. }) | (ctr @ Ctr { .. }, Var { name }) => {
        if ctr.has_var(name) {
          Err(NotUnifiable)
        } else {
          Ok(Subs::new(hashmap! { name.clone() => ctr.clone() }))
        }
      }

      (Num { val: x }, Num { val: y }) if x == y => Ok(Subs::empty()),
      (Num { .. }, Num { .. }) => Err(NotUnifiable),

      (Var { name }, num @ Num { .. }) | (num @ Num { .. }, Var { name }) => {
        Ok(Subs::new(hashmap! { name.clone() => num.clone() }))
      }

      (Ctr { name: x, args: xs }, Ctr { name: y, args: ys })
        if x != y || xs.len() != ys.len() =>
      {
        Err(NotUnifiable)
      }
      (Ctr { args: xs, .. }, Ctr { args: ys, .. }) => {
        let subs = combine(xs.clone(), ys.clone()).iter().fold(
          Ok(Subs::empty()),
          |acc, (fst, snd)| {
            let acc = acc?;
            let fst = fst.subst(&acc);
            let snd = snd.subst(&acc);
            let subs = fst.mgu(&snd)?;
            Ok(acc.compose(subs))
          },
        );
        subs
      }
      //{
      // todo!()
      // if x != y || xs.len() != ys.len() {
      //   Err(NotUnifiable)
      // } else {
      //   let subs = combine(xs.clone(), ys.clone()).iter().fold(
      //     hashmap! {},
      //     |acc, (fst, snd)| {
      //       let fst = fst.subst(&acc);
      //       let snd = snd.subst(&acc);
      //       let subs = fst.mgu(&snd).unwrap();
      //       compose(acc, subs)
      //     },
      //   );
      //   Ok(subs)
      // }
      //}
      _ => Err(NotUnifiable),
    }
  }
}

impl Mgu for Atom {
  type ErrType = UnificationError;

  fn mgu(&self, other: &Self) -> Result<Subs, Self::ErrType> {
    // let Self(sym1, args1) = self;
    // let Self(sym2, args2) = other;
    // let ctr1 = Term::Ctr { symbol: sym1.clone(), args: args1.clone() };
    // let ctr2 = Term::Ctr { symbol: sym2.clone(), args: args2.clone() };
    let ctr1 = self.as_ctr();
    let ctr2 = other.as_ctr();
    ctr1.mgu(&ctr2)
  }
}

#[cfg(test)]
mod test {
  use crate::ast::Term::*;

  use super::{Mgu, UnificationError};

  #[test]
  fn test() -> Result<(), UnificationError> {
    // let t1 = Var {
    //   name: String::from("x"),
    // };
    // let t2 = Ctr {
    //   name: String::from("f"),
    //   args: vec![],
    // };
    let t1 = Var { name: "oi".into() };
    let t2 = Num { val: 4 };
    let mgu = t1.mgu(&t2)?;
    println!("mgu: {mgu:?}");
    Ok(())
  }
}
