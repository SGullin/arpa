#[derive(Debug, Clone, Copy)]
pub enum Table {
    Users,

    PulsarMetas,
    ParMetas,
    RawMetas,
    TemplateMetas,
    
    Toas,
    
    Telescopes,
    ObsSystems,
    
    ProcessMetas,
    DiagnosticFloats,
    DiagnosticPlots,
}
impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Users => write!(f, "users"),

            Self::PulsarMetas => write!(f, "pulsar_meta"),
            Self::ParMetas => write!(f, "par_meta"),
            Self::RawMetas => write!(f, "raw_meta"),
            Self::TemplateMetas => write!(f, "template_meta"),
            
            Self::Toas => write!(f, "toas"),
            
            Self::Telescopes => write!(f, "telescopes"),
            Self::ObsSystems => write!(f, "obs_systems"),
            
            Self::ProcessMetas => write!(f, "process_meta"),
            Self::DiagnosticFloats => write!(f, "diag_floats"),
            Self::DiagnosticPlots => write!(f, "diag_plots"),
        }
    }
}

pub trait TableItem: 
    for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>
    + std::marker::Send
    + std::marker::Sync
    + std::marker::Unpin
    {
    const TABLE: Table;

    fn id(&self) -> i32;
    
    /// The columns used for insertion.
    fn insert_columns() -> &'static str;
    
    /// The values used for insertion.
    fn insert_values(&self) -> String;

    /// The values used for checking conflicts.
    fn unique_values(&self) -> String;

    /// The columns used for selection.
    fn select() -> &'static str;
}
