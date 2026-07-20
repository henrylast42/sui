// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use anyhow::Context;
use diesel::QueryableByName;
use diesel::sql_types::Text;
use sui_indexer_alt_reader::pg_reader::PgReader;
use sui_sql_macro::query;
use tokio::time;
use tracing::warn;

/// Discover pipelines that have a watermark row in the database, regardless of whether they've
/// been explicitly configured in the RPC's own config file. Retries on `retry_interval` until the
/// query succeeds -- a fresh/unreachable database is treated as a transient startup condition, not
/// a fatal misconfiguration.
pub async fn discover_pipelines(pg_reader: &PgReader, retry_interval: Duration) -> Vec<String> {
    let mut interval = time::interval(retry_interval);

    loop {
        interval.tick().await;

        match fetch(pg_reader).await {
            Ok(pipelines) => return pipelines,
            Err(e) => warn!("Failed to discover pipelines, retrying: {e:#}"),
        }
    }
}

/// Try to discover pipeline names from the `watermarks` table. `pipeline` is that table's primary
/// key, so this returns at most one entry per pipeline.
async fn fetch(pg_reader: &PgReader) -> anyhow::Result<Vec<String>> {
    let mut conn = pg_reader
        .connect()
        .await
        .context("Failed to connect to database")?;

    #[derive(QueryableByName)]
    struct Row {
        #[diesel(sql_type = Text)]
        pipeline: String,
    }

    let rows: Vec<Row> = conn
        .results(query!("SELECT pipeline FROM watermarks"))
        .await
        .context("Failed to query watermarks table")?;

    Ok(rows.into_iter().map(|r| r.pipeline).collect())
}
