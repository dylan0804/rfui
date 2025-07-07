#[derive(Debug)]
pub enum ExitCode {
    Success,
    KilledBySigint,
    GeneralError(String),
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        match code {
            ExitCode::Success => 0,
            ExitCode::GeneralError(_) => 1,
            ExitCode::KilledBySigint => 130,
        }
    }
}

impl ExitCode {
    pub fn exit(self) -> ! {
        std::process::exit(self.into())
    }
}
