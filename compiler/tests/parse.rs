use ariadne::Source;
use trove_core::tests::fixture::load_fixture_simple;
use trovec::parser::parse_program;

#[test]
fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
    for file in globwalk::glob("tests/fixtures/**/*.test")? {
        let path = file?.into_path();
        println!("running {:?}", path);
        let fixture = load_fixture_simple(path.clone())?;

        match parse_program(path.display().to_string(), &fixture.input) {
            Ok(program) => {
                let output = format!("{program:#?}");
                pretty_assertions::assert_eq!(output.trim(), fixture.expected.clone().trim());
            }
            Err(err) => {
                err.report()
                    .eprint((&path.display().to_string(), Source::from(fixture.input)))
                    .unwrap();
                panic!("Test failed: parse error in {path:?}")
            }
        }
    }

    Ok(())
}
