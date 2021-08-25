//! TypeInstantiations are used when instantiating a type. The argument list is given to the
//! type on execution.

use super::{
    ErrKind, Error, InstrKind, Instruction, Interpreter, ObjectInstance, Rename, TypeDec, TypeId,
    VarAssign,
};
use crate::instance::{Name, Size};

use std::rc::Rc;

#[derive(Clone)]
pub struct TypeInstantiation {
    type_name: TypeId,
    fields: Vec<VarAssign>,
}

impl TypeInstantiation {
    /// Create a new type instantiation and return it
    pub fn new(type_name: TypeId) -> TypeInstantiation {
        TypeInstantiation {
            type_name,
            fields: Vec::new(),
        }
    }

    /// Add an argument to the given type instantiation
    pub fn add_field(&mut self, arg: VarAssign) {
        self.fields.push(arg)
    }

    /// Return a reference to the instantiated type's name
    pub fn name(&self) -> &TypeId {
        &self.type_name
    }

    /// Return a reference to the list of fields
    pub fn fields(&self) -> &Vec<VarAssign> {
        &self.fields
    }

    /// Get the corresponding type declaration from an interpreter
    fn get_declaration(&self, interpreter: &mut Interpreter) -> Option<Rc<TypeDec>> {
        match interpreter.get_type(self.name()) {
            // get_type() return a Rc, so this clones the Rc, not the TypeId
            Some(t) => Some(t.clone()),
            // FIXME: Fix Location and input
            None => {
                interpreter.error(
                    Error::new(ErrKind::Interpreter)
                        .with_msg(format!("Cannot find type {}", self.name().id())),
                );
                None
            }
        }
    }

    /// Check if the fields received and the fields expected match
    fn check_fields_count(&self, type_dec: &TypeDec) -> Result<(), Error> {
        match self.fields().len() == type_dec.fields().len() {
            true => Ok(()),
            false => Err(Error::new(ErrKind::Interpreter).with_msg(format!(
                "Wrong number of arguments \
                    for type instantiation `{}`: Expected {}, got {}",
                self.name().id(),
                type_dec.fields().len(),
                self.fields().len()
            ))),
        }
    }

    /// Check if the type we're currently instantiating is a primitive type or not
    // FIXME: Remove later, as it should not be needed once typechecking is implemented
    fn check_primitive(&self) -> Result<(), Error> {
        match self.type_name.is_primitive() {
            true => Err(Error::new(ErrKind::Interpreter).with_msg(format!(
                "cannot instantiate primitive type `{}`",
                self.type_name.id()
            ))),
            false => Ok(()),
        }
    }
}

impl Instruction for TypeInstantiation {
    fn kind(&self) -> InstrKind {
        InstrKind::Expression(None)
    }

    fn print(&self) -> String {
        let mut base = format!("{}(", self.type_name.id());
        let mut first_arg = true;
        for arg in &self.fields {
            if !first_arg {
                base.push_str(", ");
            }

            base.push_str(&arg.print());

            first_arg = false;
        }

        format!("{})", base)
    }

    fn execute(&self, interpreter: &mut Interpreter) -> Option<ObjectInstance> {
        if let Err(e) = self.check_primitive() {
            interpreter.error(e);
            return None;
        }

        let type_dec = self.get_declaration(interpreter)?;

        if let Err(e) = self.check_fields_count(&type_dec) {
            interpreter.error(e);
            return None;
        }

        let mut size: usize = 0;
        let mut data: Vec<u8> = Vec::new();
        let mut fields: Vec<(Name, Size)> = Vec::new();
        for (_, named_arg) in self.fields.iter().enumerate() {
            // FIXME: Need to assign the correct field to the field that corresponds
            // in the typedec
            let field_instr = named_arg.value();
            let field_name = named_arg.symbol();

            // FIXME: Use execute_expression() here?
            let instance = field_instr.execute_expression(interpreter)?;

            let inst_size = instance.size();
            size += inst_size;
            fields.push((field_name.to_string(), inst_size));
            data.append(&mut instance.data().to_vec());
        }

        Some(ObjectInstance::new(
            // FIXME: Disgusting, maybe do not use Rc for TypeId?
            Some((*type_dec).clone()),
            size,
            data,
            Some(fields),
        ))
    }
}

impl Rename for TypeInstantiation {
    fn prefix(&mut self, prefix: &str) {
        self.type_name.prefix(prefix);
        // FIXME
        // self.fields
        //     .iter_mut()
        //     .for_each(|field| field.prefix(prefix));
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn t_fields_number() {
        use super::super::{DecArg, TypeId};
        use crate::value::JkInt;

        let mut interpreter = Interpreter::new();

        // Create a new type with two integers fields
        let fields = vec![
            DecArg::new("a".to_owned(), TypeId::from("int")),
            DecArg::new("b".to_owned(), TypeId::from("int")),
        ];
        let t = TypeDec::new("Type_Test".to_owned(), fields);

        t.execute(&mut interpreter);

        let mut t_inst = TypeInstantiation::new(TypeId::from("Type_Test"));

        assert!(t_inst.execute(&mut interpreter).is_none());
        assert!(
            interpreter.error_handler.has_errors(),
            "Given 0 field to 2 fields type"
        );
        interpreter.clear_errors();

        t_inst.add_field(VarAssign::new(
            false,
            "a".to_string(),
            Box::new(JkInt::from(12)),
        ));

        assert!(t_inst.execute(&mut interpreter).is_none());
        assert!(
            interpreter.error_handler.has_errors(),
            "Given 1 field to 2 fields type"
        );
        interpreter.clear_errors();

        t_inst.add_field(VarAssign::new(
            false,
            "b".to_string(),
            Box::new(JkInt::from(8)),
        ));

        assert!(
            t_inst.execute(&mut interpreter).is_some(),
            "Type instantiation should have a correct number of fields now"
        );
        assert!(!interpreter.error_handler.has_errors());
    }

    #[test]
    fn t_returned_instance() {
        use super::super::{DecArg, TypeId};
        use crate::value::{JkInt, JkString};

        const TYPE_NAME: &str = "Type_Name";

        let mut interpreter = Interpreter::new();

        // Create a new type with two integers fields
        let fields = vec![
            DecArg::new("a".to_owned(), TypeId::from("string")),
            DecArg::new("b".to_owned(), TypeId::from("int")),
        ];
        let t = TypeDec::new(TYPE_NAME.to_owned(), fields);

        t.execute(&mut interpreter);

        let mut t_inst = TypeInstantiation::new(TypeId::new(TYPE_NAME.to_string()));
        t_inst.add_field(VarAssign::new(
            false,
            "a".to_string(),
            Box::new(JkString::from("I am a loooooooong string")),
        ));
        t_inst.add_field(VarAssign::new(
            false,
            "b".to_string(),
            Box::new(JkInt::from(12)),
        ));

        let instance = match t_inst.execute(&mut interpreter) {
            Some(instance) => instance,
            None => {
                unreachable!("Type instantiation should have returned an Expression")
            }
        };

        assert!(
            instance.ty().unwrap().name() == TYPE_NAME,
            "Type name should be {}",
            TYPE_NAME
        );
        assert_eq!(
            instance.data()[..],
            [
                73, 32, 97, 109, 32, 97, 32, 108, 111, 111, 111, 111, 111, 111, 111, 111, 110, 103,
                32, 115, 116, 114, 105, 110, 103, 12, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(instance.size(), 33);

        assert_eq!(instance.fields().as_ref().unwrap().get("a"), Some(&(0, 25)));
        assert_eq!(instance.fields().as_ref().unwrap().get("b"), Some(&(25, 8)));
    }

    #[test]
    fn t_instantiate_primitive() {
        use crate::parser::Construct;

        let mut i = Interpreter::new();

        let instr = Construct::instruction("i = int { no_field = 15 }")
            .unwrap()
            .1;

        assert!(instr.execute(&mut i).is_none());
        assert!(i.error_handler.has_errors());
    }
}
