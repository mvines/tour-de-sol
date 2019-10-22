//! Ramp up TPS for Tour de SOL until all validators drop out

mod utils;

use clap::{crate_description, crate_name, crate_version, App, Arg};
use solana_client::rpc_client::RpcClient;
use solana_sdk::genesis_block::GenesisBlock;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

const TDS_ENTRYPOINT: &str = "tds.solana.com";
const TMP_LEDGER_PATH: &str = ".tmp/ledger";

fn main() {
    solana_logger::setup();

    let matches = App::new(crate_name!())
        .about(crate_description!())
        .version(crate_version!())
        .arg(
            Arg::with_name("entrypoint")
                .short("n")
                .long("entrypoint")
                .value_name("HOST")
                .takes_value(true)
                .default_value(TDS_ENTRYPOINT)
                .validator(utils::is_host)
                .help("Download the genesis block from this entry point"),
        )
        .get_matches();

    let tmp_ledger_path = PathBuf::from(TMP_LEDGER_PATH);
    let _ = fs::remove_dir_all(&tmp_ledger_path);
    fs::create_dir_all(&tmp_ledger_path).expect("failed to create temp ledger path");

    let entrypoint_str = matches.value_of("entrypoint").unwrap();
    println!("Connecting to {}", entrypoint_str);
    let entrypoint_addr = solana_netutil::parse_host_port(&format!("{}:8899", entrypoint_str))
        .expect("failed to parse entrypoint address");
    utils::download_genesis(&entrypoint_addr, &tmp_ledger_path).expect("genesis download failed");
    let genesis_block = GenesisBlock::load(&tmp_ledger_path).expect("failed to load genesis block");

    println!("Fetching current slot");
    let rpc_client = RpcClient::new_socket_with_timeout(entrypoint_addr, Duration::from_secs(10));
    let current_slot = rpc_client.get_slot().expect("failed to fetch current slot");
    let first_normal_slot = genesis_block.epoch_schedule.first_normal_slot;
    let sleep_slots = first_normal_slot.saturating_sub(current_slot);
    utils::sleep_n_slots(sleep_slots, &genesis_block);

    /* TODO
    while not dead {
       wait for stakes to fully warm up/cool down
       run bench-tps for 60min at a rate of $TPS
       for all validators in the 2/3 partition {
            gift SOL_GIFT in stake
       }
       SOL_GIFT *= 2
       TPS += N
    }
    compute prizes using the TdS winner-tool
    */
}