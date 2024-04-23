pub mod config;
pub mod errors;
pub mod ipaddr;

use std::path::PathBuf;

use aws_sdk_route53::model::{
    Change, ChangeAction, ChangeBatch, ResourceRecord, ResourceRecordSet, RrType,
};
use aws_sdk_route53::Client;
use structopt::StructOpt;

use config::parse_config;
use ipaddr::{get_ip, is_current_address};

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(short, long, env, parse(from_os_str))]
    config_file: PathBuf,
}

fn build_change_object(ip: &str, name: &str, ttl: i64) -> ChangeBatch {
    let record = ResourceRecord::builder().value(ip).build();
    let record_set = ResourceRecordSet::builder()
        .name(name)
        .r#type(RrType::A)
        .ttl(ttl)
        .resource_records(record)
        .build();
    let change = Change::builder()
        .action(ChangeAction::Upsert)
        .resource_record_set(record_set)
        .build();
    ChangeBatch::builder()
        .comment("Update IP address")
        .changes(change)
        .build()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Opt { config_file } = Opt::from_args();

    let config = parse_config(&config_file)?;

    let host_ip = get_ip(&config.ip_check).await?;
    let host_ip = host_ip.trim();
    if !config.always_update_record && is_current_address(&config.record_name, host_ip)? {
        println!("Avoiding unnecessary work. Record is already correct.");
        return Ok(());
    }
    let shared_config = aws_config::load_from_env().await;
    let client = Client::new(&shared_config);
    let batch = build_change_object(host_ip, &config.record_name, config.ttl);
    let req = client
        .change_resource_record_sets()
        .hosted_zone_id(&config.hosted_zone_id)
        .change_batch(batch);
    let resp = req.send().await?;
    println!("Request response: {:?}", resp);
    Ok(())
}
