use std::fmt::{Display, Formatter, Result};

use crate::instruction::TypeId;

#[derive(Clone, Debug, PartialEq)]
pub struct DecArg {
    name: String,
    ty: TypeId,
}

impl DecArg {
    /// Create a new function declaration argument with a name and a type
    pub fn new(name: String, ty: TypeId) -> DecArg {
        DecArg { name, ty }
    }

    /// Return a reference to the argument's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Return a representation of the argument's type
    pub fn get_type(&self) -> &TypeId {
        &self.ty
    }
}

impl Display for DecArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}: {}", self.name, self.ty.id())
    }
}
