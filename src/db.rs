use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::configuration::DatabaseSettings;

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub async fn create_foreign_connection(
    pool: &PgPool,
    configuration: &DatabaseSettings,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        CREATE EXTENSION IF NOT EXISTS postgres_fdw;
    "#
    )
    .execute(pool)
    .await?;

    sqlx::query(&format!(
        r#"
            CREATE SERVER IF NOT EXISTS fdw_server_connection 
            FOREIGN DATA WRAPPER postgres_fdw 
            OPTIONS (dbname {}, host {}, port {});
        "#,
        configuration.database_name, configuration.host, configuration.port,
    ))
    .execute(pool)
    .await?;

    sqlx::query(&format!(r#"
        CREATE USER MAPPING IF NOT EXISTS FOR postgres 
        SERVER fdw_server_connection 
        OPTIONS (user {}, password {});
        "#,
        configuration.username,
        configuration.password.expose_secret(),
    )
    ).execute(pool).await?;

    sqlx::query!(
        r#"
        IMPORT FOREIGN SCHEMA "public"
        LIMIT
                TO (users, tracks, blocks, ursm_content_nodes)
        FROM
                SERVER fdw_server_connection INTO public; 
        "#
    )
    .execute(pool)
    .await?;

    Ok(())
}
