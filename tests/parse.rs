mod fixture;
use ariadne::Source;
use fixture::load_fixture_simple;
use trovevm::cli::{
    InputSource,
    commands::{ParseCommandError, parse},
};

#[test]
fn test_parse() -> Result<(), Box<dyn std::error::Error>> {
    for file in globwalk::glob("tests/fixtures/parse/*.test")? {
        let path = file?.into_path();
        println!("running {:?}", path);
        let fixture = load_fixture_simple(path.clone())?;

        match parse(InputSource::String(fixture.input.clone())) {
            Ok(program) => {
                let output = format!("{program:#?}");
                pretty_assertions::assert_eq!(output.trim(), fixture.expected.clone().trim());
            }
            Err(err) => {
                match err {
                    ParseCommandError::Input(e) => eprintln!("{e}"),
                    ParseCommandError::Parse(e) => e
                        .report()
                        .eprint((&e.source_name, Source::from(&e.source_code)))
                        .unwrap(),
                };
                panic!("Test failed: parse error in {path:?}")
            }
        }
    }

    Ok(())
}
