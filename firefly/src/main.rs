use std::collections::HashMap;
use std::fmt::Display;

use anyhow::anyhow;
use api::{db, firefly};
use clap::{Parser, Subcommand};
use secp256k1::SecretKey;
use uuid::Uuid;

mod api;

#[derive(Debug, Parser)]
struct Args {
    /// Wallet key in hex format
    #[arg(long)]
    wallet_key: String,

    /// Firefly deploy service url
    #[arg(long)]
    deploy_service_url: String,

    /// Firefly propose service url
    #[arg(long)]
    propose_service_url: String,

    /// Globally unique service identifier
    #[arg(long)]
    service_id: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Upload db snaphot
    Upload {
        /// Postgres connection string
        #[arg(long)]
        db_url: String,
    },

    /// Download db snaphot
    Download {
        /// Block hash
        #[arg(long)]
        hash: String,
    },

    /// Initialize contract
    InitContract,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let wallet_key = hex::decode(args.wallet_key)
        .as_deref()
        .map(SecretKey::from_slice)
        .unwrap()
        .unwrap();

    let mut client = firefly::Client::new(
        wallet_key,
        args.deploy_service_url,
        args.propose_service_url,
    )
    .await?;

    match args.command {
        Commands::Upload { db_url } => {
            let channel_name = Uuid::new_v4();
            let sql = db::dump(db_url)?;

            let rho_code = rho_sql_dump_template(channel_name, sql);
            let hash = client.full_deploy(rho_code).await?;
            println!("{hash}");

            let rho_code = rho_save_hash_template(args.service_id, hash, channel_name);
            let hash = client.full_deploy(rho_code).await?;
            println!("{hash}");
        }
        Commands::Download { hash } => {
            let entries: Vec<HashMap<String, String>> = client
                .get_channel_value(hash, format!("{}-hashes", args.service_id))
                .await?;

            let Some(entry) = entries.into_iter().last() else {
                return Err(anyhow!("no data"));
            };

            let sql: String = client
                .get_channel_value(entry["block_hash"].clone(), entry["channel_name"].clone())
                .await?;
            println!("{sql}");
        }
        Commands::InitContract => {
            let rho_code = rho_save_hash_contract(args.service_id);
            let hash = client.full_deploy(rho_code).await?;
            println!("{hash}");
        }
    }

    Ok(())
}

fn rho_sql_dump_template(channel_name: impl Display, sql: String) -> String {
    format!(r#"@"{channel_name}"!("{sql}")"#)
}

fn rho_save_hash_template(
    service_id: String,
    block_hash: String,
    channel_name: impl Display,
) -> String {
    format!(
        r#"
        @"{service_id}-hash"!(
            {{
                "block_hash": "{block_hash}",
                "channel_name": "{channel_name}",
            }}
        )"#
    )
}

fn rho_save_hash_contract(service_id: String) -> String {
    format!(
        r#"
        @"{service_id}-hashes"!([])
        |
        contract @"{service_id}-hash"(data) = {{
            for(@hashes <- @"{service_id}-hashes") {{
                @"{service_id}-hashes"!(hashes ++ [*data])
            }}
        }}
        "#
    )
}
