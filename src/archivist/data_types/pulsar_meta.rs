//! Pulsar metadata.

use std::str::FromStr;

use crate::{ARPAError, Table, archivist::TableItem};

#[derive(Debug, sqlx::FromRow, Clone)]
/// Metadata of a pulsar.
pub struct PulsarMeta {
    /// Mandatory id.
    pub id: i32,

    /// What this pulsar is commonly called.
    pub alias: String,
    /// The J name, if different from the alias.
    pub j_name: Option<String>,
    /// The B name, if any and different from the alias.
    pub b_name: Option<String>,

    /// The right ascension, as a fully specified string; "HH:MM:SS.F*".
    pub j2000_ra: Option<String>,
    /// The declination, as a fully specified string; "±DD:MM:SS.F*".
    pub j2000_dec: Option<String>,

    /// The id of a master ephemeride, if it has any set.
    pub master_parfile_id: Option<i32>,
}
impl PulsarMeta {
    /// Verifies the data is valid.
    /// # Errors
    /// Each field, except for `master_partfile_id`, is verified, and any
    /// errors surface here.
    pub fn verify(&mut self) -> Result<(), ARPAError> {
        if !Self::validate_name(&self.alias) {
            return Err(ARPAError::MalformedInput(format!(
                "\"{}\" is not a valid pulsar alias",
                self.alias
            )));
        }
        if let Some(j) = &self.j_name {
            if !Self::validate_name(j) {
                return Err(ARPAError::MalformedInput(format!(
                    "\"{j}\" is not a valid pulsar J name"
                )));
            }
        }
        if let Some(b) = &self.b_name {
            if !Self::validate_name(b) {
                return Err(ARPAError::MalformedInput(format!(
                    "\"{b}\" is not a valid pulsar B name"
                )));
            }
        }
        if let Some(ra) = &self.j2000_ra {
            psrutils::data_types::J2000Ra::from_str(ra)?;
        }
        if let Some(dec) = &self.j2000_dec {
            psrutils::data_types::J2000Dec::from_str(dec)?;
        }

        if Some(&self.alias) == self.j_name.as_ref() {
            self.j_name = None;
        }
        if Some(&self.alias) == self.b_name.as_ref() {
            self.b_name = None;
        }
        self.id = 0;

        Ok(())
    }

    /// A null meta.
    /// # Notes
    /// This is not a valid entry, so it needs to be verified before insertion.
    pub const fn null() -> Self {
        Self {
            id: 0,
            alias: String::new(),
            j_name: None,
            b_name: None,
            j2000_ra: None,
            j2000_dec: None,
            master_parfile_id: None,
        }
    }

    /// Tries to read the information from a slice of `&str`s.
    ///
    /// Each line must contain an `alias`, and may contain `j_name`, `b_name`,
    /// `ra`, and `dec`, in that order.
    ///
    /// If you want to skip a field, enter a `.` in its place.
    ///
    /// # Errors
    /// Fails if the slice is empty or the verification at the end fails.
    ///
    /// # Examples
    /// ```
    /// # use argos_arpa::data_types::PulsarMeta;
    /// let line = vec!["alias", ".", "b9000+01"];
    /// let pm = PulsarMeta::from_strs(&line).unwrap();
    ///
    /// assert_eq!(pm.alias, "alias");
    /// assert_eq!(pm.j_name, None);
    /// assert_eq!(pm.b_name, Some("b9000+01".into()));
    /// ```
    pub fn from_strs(parts: &[&str]) -> Result<Self, ARPAError> {
        let mut iter = parts.iter();

        let Some(&alias) = iter.next() else {
            return Err(ARPAError::MalformedInput(
                "pulsar line is empty".into(),
            ));
        };

        let mut meta = Self::null();
        meta.alias = alias.to_string();

        let refs = vec![
            &mut meta.j_name,
            &mut meta.b_name,
            &mut meta.j2000_ra,
            &mut meta.j2000_dec,
        ];

        for r in refs {
            if let Some(&text) = iter.next() {
                if text == "." {
                    continue;
                }
                *r = Some(text.to_string());
            }
        }

        meta.verify()?;

        Ok(meta)
    }

    fn validate_name(name: &str) -> bool {
        name.chars()
            .all(|c| c.is_alphanumeric() || ['+', '-'].contains(&c))
            && !name.is_empty()
            && name.len() <= 20
    }
}
impl TableItem for PulsarMeta {
    const TABLE: Table = Table::PulsarMetas;

    fn id(&self) -> i32 {
        self.id
    }

    fn insert_values(&self) -> String {
        format!(
            "'{}', {}, {}, {}, {}, {}",
            self.alias,
            self.j_name
                .as_ref()
                .map_or_else(|| "NULL".into(), |j| format!("'{j}'")),
            self.b_name
                .as_ref()
                .map_or_else(|| "NULL".into(), |b| format!("'{b}'")),
            self.j2000_ra
                .as_ref()
                .map_or_else(|| "NULL".into(), |r| format!("'{r}'")),
            self.j2000_dec
                .as_ref()
                .map_or_else(|| "NULL".into(), |d| format!("'{d}'")),
            self.master_parfile_id
                .as_ref()
                .map_or_else(|| "NULL".into(), |m| format!("{m}")),
        )
    }

    fn insert_columns() -> &'static str {
        "alias, j_name, b_name, j2000_ra, j2000_dec, master_parfile_id"
    }

    fn select() -> &'static str {
        "id, alias, j_name, b_name, j2000_ra, j2000_dec, master_parfile_id"
    }

    fn unique_values(&self) -> String {
        self.j_name.as_ref().map_or_else(
            || format!("alias='{}'", self.alias),
            |jn| format!("alias='{}' or j_name='{}'", self.alias, jn),
        )
    }
}
impl FromStr for PulsarMeta {
    type Err = ARPAError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_strs(
            &s.split_whitespace()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>(),
        )
    }
}
