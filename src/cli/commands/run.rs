use miette::Result;

use crate::cli::input::InputSource;

pub fn run(input: InputSource) -> Result<()> {
    //let source = input.read_to_string()?;

    /*
        let program = match core::parser::parse_program(&source, input.source_name()) {
            Ok(program) => program,
            Err(error) => {
                if pretty_errors {
                    writeln!(
                        err,
                        "{:?}",
                        miette::Report::new(error).with_source_code(source)
                    )
                    .into_diagnostic()?;
                } else {
                    writeln!(err, "{}", error.simple_report()).into_diagnostic()?;
                }
                return Ok(RunResult::ParseError);
            }
        };

        let resolver = Resolver::new();
        let locals = match resolver.resolve(&program) {
            Ok(locals) => locals,
            Err(error) => {
                writeln!(err, "{error}").into_diagnostic()?;
                return Ok(RunResult::ParseError);
            }
        };
        //debug_locals(&locals, &program);

        let mut interpreter = Interpreter::new(out).with_locals(locals);
        if let Err(error) = interpreter.execute(program) {
            writeln!(err, "{error}.").into_diagnostic()?;
            return Ok(RunResult::RuntimeError);
        }
    */

    Ok(())
}
