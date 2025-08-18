use ariadne::Source;
use trove_core::tests::fixture::load_fixture_simple;

#[test]
fn test_run() -> Result<(), Box<dyn std::error::Error>> {
    for file in globwalk::glob("tests/fixtures/**/*.test")? {
        let path = file?.into_path();
        println!("running {:?}", path);
        let fixture = load_fixture_simple(path.clone())?;

        let ast = match trovec::parser::parse_program(path.display().to_string(), &fixture.input) {
            Ok(ast) => ast,
            Err(err) => {
                err.report()
                    .eprint((&path.display().to_string(), Source::from(fixture.input)))
                    .unwrap();
                panic!("Test failed: parse error in {path:?}")
            }
        };

        let compiled_program = match trovec::codegen::compile(&ast.statements) {
            Ok(function) => function,
            Err(err) => {
                eprintln!("{err}");
                panic!("Test failed: parse error in {path:?}")
            }
        };

        let mut vm = trove_vm::VM::default();
        if let Err(err) = vm.run(compiled_program.script) {
            eprintln!("{err}");
            panic!("Test failed: parse error in {path:?}")
        }
    }

    Ok(())
}
