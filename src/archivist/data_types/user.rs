//! Data of users.

use item_macro::TableItem;
use sqlx::types::time;

use crate::{ARPAError, Result, archivist::table::TableItem};

#[derive(Debug, sqlx::FromRow, TableItem)]
#[table(Users)]
/// A user on this machine.
pub struct User {
    #[derived]
    id: i32,

    #[unique]
    username: String,
    real_name: String,
    #[unique]
    email: String,
    is_admin: bool,

    created_at: time::OffsetDateTime,
}

impl User {
    /// Creates a new user object.
    /// # Errors
    /// Fails if any of `username`, `real_name`, or `email` is not valid.
    pub fn new(
        username: &str,
        real_name: &str,
        email: &str,
        admin: bool,
    ) -> Result<Self> {
        let username = Self::validate_username(username)?;
        let real_name = Self::validate_name(real_name)?;
        let email = Self::validate_email(email)?;

        Ok(Self {
            id: 0,
            username,
            real_name,
            email,
            is_admin: admin,
            created_at: time::OffsetDateTime::now_utc(),
        })
    }

    fn validate_username(name: &str) -> Result<String> {
        if name.len() > 12 || name.len() < 3 {
            return Err(ARPAError::MalformedInput(format!(
                "'{name}'; username must be 3--12 characters long."
            )));
        }
        if !name.is_ascii() {
            return Err(ARPAError::MalformedInput(format!(
                "'{name}'; username must be only ASCII."
            )));
        }
        if name.contains(|c: char| c.is_ascii_whitespace()) {
            return Err(ARPAError::MalformedInput(format!(
                "'{name}'; username cannot contain whitespace."
            )));
        }

        Ok(name.to_ascii_lowercase())
    }

    fn validate_name(name: &str) -> Result<String> {
        if name.len() < 3 {
            return Err(ARPAError::MalformedInput(format!(
                "'{name}'; name must be at over 2 characters long."
            )));
        }

        Ok(name.to_string())
    }

    /// Some very basic email checking.
    fn validate_email(email: &str) -> Result<String> {
        let at_pos = email.find('@').ok_or(ARPAError::MalformedInput(
            format!("'{email}'; Email addresses need an @"),
        ))?;
        email.rfind('.').filter(|p| *p > at_pos).ok_or(
            ARPAError::MalformedInput(format!(
                "'{email}'; Email addresses need a domain"
            )),
        )?;

        Ok(email.to_string())
    }

    /// The time this user was created (in the host's timekeeping system).
    pub const fn created_at(&self) -> time::OffsetDateTime {
        self.created_at
    }
}
