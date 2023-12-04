use std::fmt;

use crate::ast::Term;

impl fmt::Display for Term {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Term::Var { name } => write!(f, "{name}"),
      Term::Num { val } => write!(f, "{val}"),
      Term::Ctr { name, args } => {
        let args = args
          .iter()
          .map(|t| t.to_string())
          .collect::<Vec<_>>()
          .join(", ");
        write!(f, "{name}({args})")
      }
    }
  }
}
