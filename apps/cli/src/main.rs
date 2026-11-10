use rsphysics_catalog::official_catalog;
use rsphysics_model_api::{Backend, ErrorKind, ModelError, parse_model_manifest};
use rsphysics_runtime::{BuildInfo, Runtime};
use std::{env, fs, process::ExitCode};

const HELP: &str = "RSPhysics\n\nrsphysics models list\nrsphysics models describe <model-id>\nrsphysics manifest validate <model.toml>\nrsphysics run --model <model-id> --input <parameters.json>\n\nResults: JSON on stdout. Errors: JSON on stderr, exit code 1.";

fn execute(args: &[String]) -> Result<(), ModelError> {
    let runtime = Runtime::new(official_catalog()?, BuildInfo::default(), Backend::Native);
    let output = match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [] | ["--help"] | ["-h"] => {
            println!("{HELP}");
            return Ok(());
        }
        ["--version"] => {
            println!("rsphysics {}", env!("CARGO_PKG_VERSION"));
            return Ok(());
        }
        ["models", "list"] => serde_json::to_value(runtime.list_models()),
        ["models", "describe", id] => serde_json::to_value(runtime.describe(id)?),
        ["manifest", "validate", path] => {
            let text = fs::read_to_string(path)
                .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
            serde_json::to_value(parse_model_manifest(&text)?)
        }
        ["run", "--model", id, "--input", path] => {
            let input = fs::read_to_string(path).map_err(|e| {
                ModelError::new(
                    ErrorKind::InvalidParameter,
                    format!("cannot read input: {e}"),
                )
            })?;
            let params = serde_json::from_str(&input)
                .map_err(|e| ModelError::new(ErrorKind::InvalidParameter, e.to_string()))?;
            serde_json::to_value(runtime.run(id, &params)?)
        }
        _ => return Err(ModelError::new(ErrorKind::InvalidParameter, HELP)),
    }
    .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|e| ModelError::new(ErrorKind::Internal, e.to_string()))?
    );
    Ok(())
}

fn main() -> ExitCode {
    match execute(&env::args().skip(1).collect::<Vec<_>>()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!(
                "{}",
                serde_json::to_string(&error).expect("ModelError is serializable")
            );
            ExitCode::FAILURE
        }
    }
}
