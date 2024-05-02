pub mod config;
pub mod errors;
pub mod ipaddr;

use std::path::PathBuf;
use std::time::Duration;

use aws_sdk_route53::client::Waiters;
use aws_sdk_route53::error::BuildError;
use aws_sdk_route53::types::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use aws_sdk_route53::Client;
use clap::{value_parser, Arg, ArgAction, Command};

use config::parse_config;
use ipaddr::{get_ip, is_current_address};

const MAX_WAIT_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Debug)]
struct Opt {
    config_file: PathBuf,
}

fn parse_args() -> Opt {
    let cmd = Command::new("dynamic-route53").arg(
        Arg::new("config-file")
            .short('C')
            .long("config-file")
            .action(ArgAction::Set)
            .value_name("FILE")
            .value_parser(value_parser!(std::path::PathBuf))
            .env("CONFIG_FILE")
            .required(true),
    );

    let matches = cmd.get_matches();
    Opt {
        config_file: matches.get_one::<PathBuf>("config-file").unwrap().clone(),
    }
}

fn build_change_object(ip: &str, name: &str, ttl: i64) -> Result<ChangeBatch, BuildError> {
    let record = ResourceRecord::builder().value(ip).build()?;
    let record_set = ResourceRecordSet::builder()
        .name(name)
        .r#type(RrType::A)
        .ttl(ttl)
        .resource_records(record)
        .build()?;
    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(record_set)
        .build()?;
    ChangeBatch::builder()
        .comment("Update IP address")
        .changes(change)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Opt { config_file } = parse_args();

    let config = parse_config(&config_file)?;

    let host_ip = get_ip(&config.ip_check).await?;
    let host_ip = host_ip.trim();
    if !config.always_update_record && is_current_address(&config.record_name, host_ip)? {
        println!("Avoiding unnecessary work. Record is already correct.");
        return Ok(());
    }
    let shared_config = aws_config::from_env().load().await;
    let client = Client::new(&shared_config);
    let batch = build_change_object(host_ip, &config.record_name, config.ttl)?;
    let req = client
        .change_resource_record_sets()
        .hosted_zone_id(&config.hosted_zone_id)
        .change_batch(batch);
    let resp = req.send().await?;
    println!("Request response: {:?}", resp);

    let change_id = resp
        .change_info()
        .map(|info| info.id())
        .ok_or("No change info")?;
    let result = client
        .wait_until_resource_record_sets_changed()
        .id(change_id)
        .wait(MAX_WAIT_TIMEOUT)
        .await?;

    println!("Result after waiting: {:?}", result);
    Ok(())
}
