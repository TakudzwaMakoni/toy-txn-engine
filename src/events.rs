use std::fmt::Display;
// errors which occur during processing
#[derive(Debug, PartialEq, Clone)]
pub enum ProcessEvent {
    ProcessComplete,
    ExternalErr(String),
}

impl Display for ProcessEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessEvent::ProcessComplete => write!(f, "",),
            ProcessEvent::ExternalErr(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for ProcessEvent {}
