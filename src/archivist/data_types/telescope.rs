//! Telescope and observation information.

use crate::{ARPAError, Archivist, Result, TableItem};
use item_macro::TableItem;

#[derive(sqlx::FromRow, TableItem)]
#[table(Telescopes)]
/// Identifier of a telescope.
pub struct TelescopeId {
    #[derived]
    id: i32,

    #[unique]
    name: String,
    abbreviation: String,
    #[unique]
    code: String,
}

#[derive(sqlx::FromRow, TableItem)]
#[table(ObsSystems)]
/// An observation system.
pub struct ObsSystem {
    /// Mandatory id.
    #[derived]
    pub id: i32,

    #[unique]
    name: String,
    telescope_id: i32,
    frontend: String,
    backend: String,
    clock: String,
    code: String,
}
impl ObsSystem {
    /// Tries to find an `ObsSystem` from the DB.
    /// # Errors
    /// Fails if the name cannot be normalised.
    pub async fn find(
        archivist: &Archivist,
        name: &str,
        receiver: &str,
        backend: &str,
    ) -> Result<Option<Self>> {
        // Normalise name
        let telescope = archivist
            .find::<TelescopeId>(&format!(
                "name='{0}' or abbreviation='{0}'",
                name.to_lowercase(),
            ))
            .await?
            .ok_or(ARPAError::CantFind(format!(
                "Telescope with name or abbreviation '{name}'"
            )))?;

        let finding = archivist
            .find(&format!(
                "telescope_id={} and frontend='{}' and backend='{}'",
                telescope.id,
                receiver.to_lowercase(),
                backend.to_ascii_lowercase(),
            ))
            .await?;

        Ok(finding)
    }
}
