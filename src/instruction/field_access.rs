//! FieldAccesses represent an access onto a type instance's members.
//! FIXME: Add doc

use crate::{Context, ErrKind, Error, InstrKind, Instruction, ObjectInstance};

#[derive(Clone)]
pub struct FieldAccess {
    instance: Box<dyn Instruction>,
    field_name: String,
}

impl FieldAccess {
    /// Create a new field access from the instance accessed and the field seeked
    pub fn new(instance: Box<dyn Instruction>, field_name: String) -> FieldAccess {
        FieldAccess {
            instance,
            field_name,
        }
    }
}

impl Instruction for FieldAccess {
    fn kind(&self) -> InstrKind {
        // A field access can only ever be an expression, since we cannot store statements
        // in a type
        InstrKind::Expression(None)
    }

    fn print(&self) -> String {
        format!("{}.{}", self.instance.print(), self.field_name)
    }

    fn execute(&self, ctx: &mut Context) -> Option<ObjectInstance> {
        ctx.debug("FIELD ACCESS ENTER", &self.print());

        let calling_instance = match self.instance.execute(ctx) {
            None => {
                ctx.error(Error::new(ErrKind::Context).with_msg(format!(
                    "instance `{}` is a statement and cannot be accessed",
                    self.instance.print()
                )));
                return None;
            }
            Some(i) => i,
        };
        let field_instance = match calling_instance.get_field(&self.field_name) {
            Ok(field) => field,
            Err(e) => {
                ctx.error(e);
                return None;
            }
        };

        ctx.debug("FIELD ACCESS EXIT", &self.print());

        Some(field_instance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::ToObjectInstance;
    use crate::jinko;
    use crate::parser::Construct;
    use crate::JkInt;

    fn setup() -> Context {
        let ctx = jinko! {
            type Point(x: int, y:int);
            func basic() -> Point { Point { x = 15, y = 14 }}
            b = basic();
        };

        ctx
    }

    #[test]
    fn t_valid_field_access() {
        let mut ctx = setup();

        let inst = Construct::instruction("b.x").unwrap().1;
        let res = match inst.execute(&mut ctx) {
            Some(i) => i,
            None => return assert!(false, "Error when accessing valid field"),
        };

        let exp = JkInt::from(15).to_instance();

        assert_eq!(res, exp)
    }

    #[test]
    fn t_valid_field_access_from_type_instantiation() {
        let mut ctx = setup();

        let inst = Construct::instruction("Point { x = 1, y = 2 }.x")
            .unwrap()
            .1;
        let res = match inst.execute(&mut ctx) {
            Some(i) => i,
            None => unreachable!("Error when accesing valid field"),
        };

        let exp = JkInt::from(1).to_instance();

        assert_eq!(res, exp)
    }

    #[test]
    #[ignore] // FIXME: Do not ignore once we can type instance fields
    fn t_valid_multi_field_access() {
        let mut ctx = Context::new();

        let inst = Construct::instruction("type Pair1(x: int, y: int)")
            .unwrap()
            .1;
        inst.execute(&mut ctx).unwrap();

        let inst = Construct::instruction("type Pair2(x: Pair1, y: int)")
            .unwrap()
            .1;
        inst.execute(&mut ctx).unwrap();

        let inst = Construct::instruction("p = Pair2 { x = Pair1 { x = 1, y = 2}, y = 3}")
            .unwrap()
            .1;
        inst.execute(&mut ctx).unwrap();

        let inst = Construct::instruction("p.x.y").unwrap().1;
        let res = match inst.execute(&mut ctx) {
            Some(i) => i,
            None => unreachable!("Error when accessing valid multi field"),
        };

        let mut expected = JkInt::from(2).to_instance();
        // FIXME: Remove once typechecking is implemented
        expected.set_ty(None);

        assert_eq!(res, expected)
    }

    #[test]
    fn t_field_access_on_void() {
        let mut ctx = setup();

        let inst = Construct::instruction("func void() {}").unwrap().1;
        inst.execute(&mut ctx);

        let inst = Construct::instruction("void().field").unwrap().1;
        assert!(inst.execute(&mut ctx).is_none());
        assert!(ctx.error_handler.has_errors())
    }

    #[test]
    fn t_field_access_unknown_field() {
        let mut ctx = setup();

        let inst = Construct::instruction("b.not_a_field").unwrap().1;
        assert!(inst.execute(&mut ctx).is_none());
        assert!(ctx.error_handler.has_errors());
    }

    #[test]
    fn t_field_access_field_on_primitive() {
        let mut ctx = setup();

        let inst = Construct::instruction("i = 12").unwrap().1;
        inst.execute(&mut ctx);

        let inst = Construct::instruction("i.field_on_primitive").unwrap().1;
        assert!(inst.execute(&mut ctx).is_none());
        assert!(ctx.error_handler.has_errors())
    }
}
