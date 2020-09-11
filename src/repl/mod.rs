//! The REPL module implements an interactive mode for the broccoli interpreter. You can
//! use it as is, or run a file and then enter the interactive mode.

use linefeed::{Interface, ReadResult};

use crate::error::JinkoError;
use crate::interpreter::Interpreter;
use crate::parser::Construct;

/// Empty struct for the Repl methods
pub struct Repl;

impl Repl {
    /// Parse a new input, adding it to an existing interpreter
    fn parse_reentrant<'i>(
        interpreter: &mut Interpreter,
        input: &'i str,
    ) -> Result<(), JinkoError> {
        let (_, fc) = Construct::function_call(input).unwrap();

        interpreter.entry_point.add_instruction(Box::new(fc))?;

        Ok(())
    }

    /// Launch the REPL
    pub fn launch_repl<'i>() -> Result<(), JinkoError> {
        let line_reader = Interface::new("broccoli")?;
        let mut interpreter = Interpreter::new();

        // FIXME: Add actual prompt
        line_reader.set_prompt("broccoli > ")?;

        while let ReadResult::Input(input) = line_reader.read_line()? {
            Repl::parse_reentrant(&mut interpreter, &input)?;
        }

        Ok(())
    }
}
