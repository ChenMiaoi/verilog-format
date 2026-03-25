use std::process::ExitCode;

fn main() -> ExitCode {
    match verilog_format::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}
