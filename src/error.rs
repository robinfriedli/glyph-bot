use thiserror::Error;

#[allow(clippy::enum_variant_names)]
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error("Could not establish database connection: {0}")]
    DatabaseConnectionError(String),
    #[error("There has been an error executing a query: '{0}'")]
    QueryError(diesel::result::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Self {
        Self::QueryError(e)
    }
}
