#[derive(Debug, Clone, Copy)]
#[allow(missing_docs)]
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
impl Table {
    /// A static `&str` for the name of the table.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Users => "users",

            Self::PulsarMetas => "pulsar_meta",
            Self::ParMetas => "par_meta",
            Self::RawMetas => "raw_meta",
            Self::TemplateMetas => "template_meta",

            Self::Toas => "toas",

            Self::Telescopes => "telescopes",
            Self::ObsSystems => "obs_systems",

            Self::ProcessMetas => "process_meta",
            Self::DiagnosticFloats => "diag_floats",
            Self::DiagnosticPlots => "diag_plots",
        }
    }
}
impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// This trait is what makes the `archivist` work. Any struct expected to go
/// into an `sql` table needs to implement this.
pub trait TableItem:
    for<'a> sqlx::FromRow<'a, sqlx::postgres::PgRow>
    + std::marker::Send
    + std::marker::Sync
    + std::marker::Unpin
{
    /// Which table we go into.
    const TABLE: Table;

    /// Our id.
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
