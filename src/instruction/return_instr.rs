//! Represents the usage of a variable, for example when returning from
//! a block. In jinko, variables cannot be uninitialized. Therefore, there is no
//! need to keep an option of an instance. A variable is either there, fully initialized,
//! or it's not.

use crate::{
    InstrKind, Instruction, Interpreter, JkBool, JkErrKind, JkError, ObjectInstance, Rename,
};

#[derive(Clone)]
pub struct Return {
    value: Option<Box<dyn Instruction>>,
}

impl Return {
    /// Create a new return with a potential value to be returned
    pub fn new(ret_value: Option<Box<dyn Instruction>>) -> Return {
        Return { value: ret_value }
    }
}

impl Instruction for Return {
    fn kind(&self) -> InstrKind {
        InstrKind::Expression(None)
    }

    fn print(&self) -> String {
        // format!(
        //     "{} /* : {} = {} */",
        //     self.name.clone(),
        //     self.instance.ty().unwrap_or(&"".to_owned()),
        //     self.instance
        // )

        String::from("NOT IMPLEMENTED")
    }

    fn execute(&self, interpreter: &mut Interpreter) -> Result<InstrKind, JkError> {
        Err(JkError::new(
            JkErrKind::Interpreter,
            String::from("Execution of return is not implemented yet"),
            None,
            String::from(""),
        ))
    }
}

impl Rename for Return {
    fn prefix(&mut self, prefix: &str) {}
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::value::JkInt;
//     use crate::ToObjectInstance;
//
//     #[test]
//     fn keep_instance() {
//         let mut i = Interpreter::new();
//         let mut v = Var::new("a".to_string());
//
//         let instance = JkInt::from(15).to_instance();
//         v.set_instance(instance.clone());
//
//         i.add_variable(v.clone()).unwrap();
//
//         assert_eq!(
//             v.execute(&mut i).unwrap(),
//             InstrKind::Expression(Some(instance))
//         );
//     }
// }
