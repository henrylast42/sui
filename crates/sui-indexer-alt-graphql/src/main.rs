// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use clap::Parser;
use prometheus::Registry;
use sui_futures::service::Error;
use sui_indexer_alt_graphql::args::Args;
use sui_indexer_alt_graphql::args::Command;
use sui_indexer_alt_graphql::config::RpcLayer;
use sui_indexer_alt_graphql::discover_pipelines;
use sui_indexer_alt_graphql::start_rpc;
use sui_indexer_alt_metrics::MetricsService;
use sui_indexer_alt_metrics::uptime;
use sui_indexer_alt_reader::pg_reader::PgReader;
use telemetry_subscribers::TelemetryConfig;
use tokio::fs;

// Define the `GIT_REVISION` const
bin_version::git_revision!();

static VERSION: &str = const_str::concat!(
    env!("CARGO_PKG_VERSION_MAJOR"),
    ".",
    env!("CARGO_PKG_VERSION_MINOR"),
    ".",
    env!("CARGO_PKG_VERSION_PATCH"),
    "-",
    GIT_REVISION
);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Enable tracing, configured by environment variables.
    let _guard = TelemetryConfig::new()
        // ErrorLayer is disabled by default in TelemetryConfig, but enabled by default in GraphQL
        // to give useful error output for debugging request timeouts.
        .with_enable_error_layer(true)
        .with_env()
        .init();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install CryptoProvider");

    match args.command {
        Command::Rpc {
            database_url,
            fullnode_args,
            db_args,
            kv_args,
            consistent_reader_args,
            rpc_args,
            system_package_task_args,
            metrics_args,
            config,
            subscription_args,
        } => {
            let layer = if let Some(path) = config {
                let contents = fs::read_to_string(path)
                    .await
                    .context("Failed to read configuration TOML file")?;

                toml::from_str(&contents).context("Failed to parse configuration TOML file")?
            } else {
                RpcLayer::default()
            };

            let registry = Registry::new_custom(Some("graphql_alt".into()), None)
                .context("Failed to create Prometheus registry.")?;

            let discovery_reader = PgReader::new(
                Some("graphql_pipeline_discovery"),
                Some(database_url.clone()),
                db_args.clone(),
                &registry,
            )
            .await
            .context("Failed to set up database reader for pipeline discovery")?;

            let retry_interval = layer.watermark_polling_interval();

            let discovered_pipelines = discover_pipelines(&discovery_reader, retry_interval).await;

            let rpc_config = layer.finish(discovered_pipelines);

            let pg_pipelines: Vec<String> =
                rpc_config.pipeline.pipelines().map(str::to_owned).collect();

            let metrics = MetricsService::new(metrics_args, registry);

            metrics
                .registry()
                .register(uptime(VERSION)?)
                .context("Failed to register uptime metric.")?;

            let s_rpc = start_rpc(
                Some(database_url),
                fullnode_args,
                db_args,
                kv_args,
                consistent_reader_args,
                rpc_args,
                system_package_task_args,
                subscription_args,
                VERSION,
                rpc_config,
                pg_pipelines,
                metrics.registry(),
            )
            .await?;

            let s_metrics = metrics.run().await?;

            match s_rpc.attach(s_metrics).main().await {
                Ok(()) | Err(Error::Terminated) => {}

                Err(Error::Aborted) => {
                    std::process::exit(1);
                }

                Err(Error::Task(_)) => {
                    std::process::exit(2);
                }
            }
        }

        Command::GenerateConfig => {
            let config = RpcLayer::example();
            let config_toml = toml::to_string_pretty(&config)
                .context("Failed to serialize default configuration to TOML.")?;

            println!("{config_toml}");
        }
    }

    Ok(())
}
