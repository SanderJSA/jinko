// FIXME: Add #![warn(missing_docs)]

mod args;
mod repl;

use colored::Colorize;
use jinko::{
    CheckedType, Context, ErrKind, Error, FromObjectInstance, JkBool, JkFloat, JkInt,
    ObjectInstance,
};

use args::Args;
use repl::Repl;
use std::{fs, path::Path};

// FIXME: Add documentation
pub type InteractResult = Result<(Option<ObjectInstance>, Context), Error>;

fn handle_exit_code(result: Option<ObjectInstance>) -> ! {
    use std::process::exit;

    match result {
        // A statement that completes succesfully returns 0
        None => exit(0),

        // FIXME: Maybe return different stuff based on more types?

        // If it's an expression, return if you can (if it's an int)
        Some(i) => match i.ty() {
            CheckedType::Resolved(ty) => match ty.id() {
                "int" => exit(JkInt::from_instance(&i).rust_value() as i32),
                "float" => exit(JkFloat::from_instance(&i).rust_value() as i32),
                "bool" => {
                    let b_value = JkBool::from_instance(&i).rust_value();
                    match b_value {
                        true => exit(0),
                        false => exit(1),
                    }
                }
                _ => exit(0),
            },
            CheckedType::Void => exit(0),
            CheckedType::Unknown => unreachable!("this shouldn't happen"),
        },
    }
}

fn run_tests(ctx: &mut Context) -> Result<Option<ObjectInstance>, Error> {
    let res = ctx.execute()?;
    let tests: Vec<String> = ctx
        .tests()
        .iter()
        .map(|(k, _)| k.into())
        .filter(|test_name| {
            if !ctx.args().is_empty() {
                ctx.args().contains(test_name)
            } else {
                true
            }
        })
        .collect();

    for test_name in tests {
        eprint!(
            "[{}] running test `{}`... ",
            "WTNG".yellow().blink(),
            test_name
        );
        let fdec = ctx.tests().get(&test_name).unwrap().clone();
        let _test_result = fdec.run(ctx);
        eprintln!("\r[ {} ] running test `{}`... ", "OK".green(), test_name);

        // FIXME: We should think about handling error values in tests
        // match test_result.unwrap().ty() {
        //     CheckedType::Resolved(ty) => match ty.id() {
        //         "Ok" => eprintln!("{}", "OK".green()),
        //         "Err" => eprintln!("{}", "KO".red()),
        //         _ => unreachable!(), // FIXME:
        //     },
        //     _ => unreachable!(), // FIXME:
        //     }
    }

    ctx.emit_errors();

    Ok(res)
}

fn handle_input(args: &Args, file: &Path) -> InteractResult {
    let input = fs::read_to_string(file)?;

    let mut ctx = Context::new();

    if !args.nostdlib() {
        ctx.init_stdlib()?;
    }

    jinko::parse(&mut ctx, &input)?;

    ctx.set_path(Some(file.to_owned()));
    ctx.set_args(args.project_args());
    ctx.set_debug(args.debug());

    ctx.emit_errors();
    ctx.clear_errors();

    match args.test() {
        false => match args.interactive() {
            true => Repl::new(args)?.with_context(ctx).launch(),
            false => {
                let res = ctx.execute()?;
                ctx.emit_errors();

                Ok((res, ctx))
            }
        },
        true => {
            if args.interactive() {
                return Err(Error::new(ErrKind::Context)
                    .with_msg(String::from("cannot run tests in interactive mode")));
            }

            let res = run_tests(&mut ctx)?;

            Ok((res, ctx))
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::handle();

    let result = args.input().map_or_else(
        || Repl::new(&args)?.launch(),
        |filename| handle_input(&args, filename),
    )?;

    handle_exit_code(result.0)
}
