use std::{fs, io};

use diesel::{ConnectionError, ConnectionResult};
use diesel_async::{pooled_connection::{deadpool::Pool, AsyncDieselConnectionManager, ManagerConfig}, AsyncPgConnection};
use dotenvy::dotenv;
use futures::{future::BoxFuture, FutureExt};
use lazy_static::lazy_static;
use rustls::pki_types::CertificateDer;

lazy_static! {
    pub static ref DATABASE_URL: String = std::env::var("GLYPH_DATABASE_URL").expect(
        "Missing environment variable GLYPH_DATABASE_URL must be set to connect to postgres"
    );
    pub static ref PG_ENABLE_SSL: bool = std::env::var("GLYPH_PG_ENABLE_SSL")
        .map(|val| val
            .parse::<bool>()
            .expect("GLYPH_PG_ENABLE_SSL is not a valid boolean"))
        .unwrap_or_default();
    pub static ref PG_SSL_CERT_PATH: Option<String> =
        std::env::var("GLYPH_PG_SSL_CERT_PATH").ok();
    pub static ref CONNECTION_POOL: Pool<AsyncPgConnection> = {
        let database_connection_manager = if *PG_ENABLE_SSL {
            let mut config = ManagerConfig::default();
            config.custom_setup = Box::new(establish_pg_ssl_connection);
            AsyncDieselConnectionManager::<AsyncPgConnection>::new_with_config(
                DATABASE_URL.clone(),
                config,
            )
        } else {
            AsyncDieselConnectionManager::<AsyncPgConnection>::new(DATABASE_URL.clone())
        };
        let max_db_connections = std::env::var("GLYPH_MAX_DB_CONNECTIONS")
            .unwrap_or_else(|_| String::from("10"))
            .parse::<usize>()
            .expect("GLYPH_MAX_DB_CONNECTIONS is not a valid usize");
        Pool::builder(database_connection_manager)
            .max_size(max_db_connections)
            .build()
            .expect("Failed to initialise connection pool")
    };
}

fn main() {
    dotenvy::from_path(
        std::env::current_dir()
            .map(|wd| wd.join(".env.local"))
            .unwrap(),
    )
    .ok();
    dotenvy::from_path(
        std::env::current_dir()
            .map(|wd| wd.join(".env.secret"))
            .unwrap(),
    )
    .ok();
    dotenv().ok();

    lazy_static::initialize(&CONNECTION_POOL);

    setup_logger();
}

// enable TLS for AsyncPgConnection, see https://github.com/weiznich/diesel_async/blob/main/examples/postgres/pooled-with-rustls

fn establish_pg_ssl_connection(config: &str) -> BoxFuture<ConnectionResult<AsyncPgConnection>> {
    let fut = async {
        // We first set up the way we want rustls to work.
        let rustls_config = rustls::ClientConfig::builder()
            .with_root_certificates(root_certs())
            .with_no_client_auth();
        let tls = tokio_postgres_rustls::MakeRustlsConnect::new(rustls_config);
        let (client, conn) = tokio_postgres::connect(config, tls)
            .await
            .map_err(|e| ConnectionError::BadConnection(e.to_string()))?;
        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("Database connection: {e}");
            }
        });
        AsyncPgConnection::try_from(client).await
    };
    fut.boxed()
}

fn root_certs() -> rustls::RootCertStore {
    let mut roots = rustls::RootCertStore::empty();
    let certs =
        rustls_native_certs::load_native_certs().expect("Failed to load native certificates");
    roots.add_parsable_certificates(certs);
    if let Some(ref pg_ssl_cert_path) = *PG_SSL_CERT_PATH {
        let certs =
            load_certs(pg_ssl_cert_path.as_str()).expect("Failed to load pg ssl certificate");
        roots.add_parsable_certificates(certs);
    }
    roots
}

fn load_certs(cert_path: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    let certfile = fs::File::open(cert_path)?;
    let mut reader = io::BufReader::new(certfile);

    let certs = rustls_pemfile::certs(&mut reader);
    certs.collect()
}

fn setup_logger() {
    // create logs dir as fern does not appear to handle that itself
    if !std::path::Path::new("logs/").exists() {
        std::fs::create_dir("logs").expect("Failed to create logs/ directory");
    }

    let logging_level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}]{}[{}] {}",
                record.level(),
                chrono::Local::now().format("[%Y-%m-%d %H:%M:%S]"),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("filebroker", logging_level)
        .level_for("filebroker_server", logging_level)
        .chain(std::io::stdout())
        .chain(fern::DateBased::new("logs/", "logs_%Y-%m-%d.log"))
        .apply()
        .expect("Failed to set up logging");
}
