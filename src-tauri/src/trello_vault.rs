use crate::{error::AppError, trello_api::Credentials, trello_domain::error};
pub trait Vault {
    fn load(&self) -> Result<Option<Credentials>, AppError>;
    fn save(&self, c: &Credentials) -> Result<(), AppError>;
    fn remove(&self) -> Result<(), AppError>;
}
pub struct WindowsVault;
impl WindowsVault {
    fn entry() -> Result<keyring::Entry, AppError> {
        #[cfg(not(target_os = "windows"))]
        {
            return Err(error("TRELLO_VAULT"));
        }
        #[cfg(target_os = "windows")]
        {
            keyring::Entry::new("br.com.gmad.socialplanner.trello", "personal-read-only")
                .map_err(|_| error("TRELLO_VAULT"))
        }
    }
}
impl Vault for WindowsVault {
    fn load(&self) -> Result<Option<Credentials>, AppError> {
        match Self::entry()?.get_password() {
            Ok(s) => {
                let c: Credentials = serde_json::from_str(&s).map_err(|_| error("TRELLO_VAULT"))?;
                c.validate()?;
                Ok(Some(c))
            }
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(error("TRELLO_VAULT")),
        }
    }
    fn save(&self, c: &Credentials) -> Result<(), AppError> {
        c.validate()?;
        Self::entry()?
            .set_password(&serde_json::to_string(c).map_err(|_| error("TRELLO_VAULT"))?)
            .map_err(|_| error("TRELLO_VAULT"))
    }
    fn remove(&self) -> Result<(), AppError> {
        match Self::entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(error("TRELLO_VAULT")),
        }
    }
}
