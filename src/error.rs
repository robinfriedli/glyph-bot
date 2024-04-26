use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug)]
pub enum Error {
    #[error("Could not establish database connection: {0}")]
    DatabaseConnectionError(String),
    #[error("There has been an error executing a query: '{0}'")]
    QueryError(diesel::result::Error),
    #[error("There has been an error executing a serenity request: '{0}'")]
    SerenityError(serenity::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::QueryError(e)
    }
}

impl From<serenity::Error> for Error {
    fn from(e: serenity::Error) -> Self {
        Self::SerenityError(e)
    }
}
