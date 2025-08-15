mod fixture;
use ariadne::Source;
use fixture::load_fixture_simple;
use trovevm::cli::{
    InputSource,
    commands::{RunError, run},
};

#[test]
fn test_run() -> Result<(), Box<dyn std::error::Error>> {
    for file in globwalk::glob("tests/fixtures/run/**/*.test")? {
        let path = file?.into_path();
        println!("running {:?}", path);
        let fixture = load_fixture_simple(path.clone())?;

        match run(InputSource::String(fixture.input.clone())) {
            Ok(result) => {
                let output = format!("{result}");
                pretty_assertions::assert_eq!(output.trim(), fixture.expected.clone().trim());
            }
            Err(err) => {
                match err {
                    RunError::Input(e) => eprintln!("{e}"),
                    RunError::Parse(e) => e
                        .report()
                        .eprint((&e.source_name, Source::from(&e.source_code)))
                        .unwrap(),
                    RunError::Compile(e) => eprintln!("{e}"),
                    RunError::Runtime(e) => eprintln!("{e}"),
                };
                panic!("Test failed: parse error in {path:?}")
            }
        }
    }

    Ok(())
}
