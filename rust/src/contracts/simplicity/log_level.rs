
/// Log level for Simplicity program execution tracing.
#[derive(Clone, Copy, Debug, Default)]
pub enum SimplicityLogLevel {
    #[default]
    None,
    Debug,
    Warning,
    Trace,
}

impl From<SimplicityLogLevel> for lwk_simplicity::simplicityhl::tracker::TrackerLogLevel {
    fn from(level: SimplicityLogLevel) -> Self {
        match level {
            SimplicityLogLevel::None => Self::None,
            SimplicityLogLevel::Debug => Self::Debug,
            SimplicityLogLevel::Warning => Self::Warning,
            SimplicityLogLevel::Trace => Self::Trace,
        }
    }
}
