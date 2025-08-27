use std::path::PathBuf;

pub struct Fixture {
    pub input: String,
    pub expected: String,
}

pub struct LineFixture {
    pub input_lines: Vec<String>,
    pub expected_lines: Vec<String>,
}

pub fn load_fixture_simple<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<Fixture, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let (input, expected) = content
        .split_once("==EXPECTED==")
        .ok_or("Missing ==EXPECTED== separator")?;

    Ok(Fixture {
        input: input.replace("==INPUT==", "").trim().to_string(),
        expected: expected.trim().to_string(),
    })
}

pub fn load_fixture_linewise<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<LineFixture, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    let mut input_mode = false;
    let mut input_lines = Vec::new();
    let mut expected_lines = Vec::new();

    for line in content.lines() {
        match line.trim() {
            "==INPUT==" => input_mode = true,
            "==EXPECTED==" => input_mode = false,
            other if input_mode => input_lines.push(other.to_string()),
            other => {
                if !other.is_empty() {
                    expected_lines.push(other.to_string());
                }
            }
        }
    }

    Ok(LineFixture {
        input_lines,
        expected_lines,
    })
}

fn _load_fixture<P: Into<PathBuf>>(path: P) -> Result<Fixture, Box<dyn std::error::Error>> {
    let path = path.into();
    let content = std::fs::read_to_string(path)?;

    let mut input = String::new();
    let mut expected = String::new();
    let mut input_mode = true;

    for line in content.lines() {
        match line {
            "==INPUT==" => {
                input_mode = true;
            }
            "==EXPECTED==" => {
                input_mode = false;
            }
            _ => {
                if input_mode {
                    input.push_str(line);
                    input.push('\n');
                } else {
                    let line = line.trim();
                    expected.push_str(line);
                    expected.push('\n');
                }
            }
        }
    }

    Ok(Fixture { input, expected })
}
