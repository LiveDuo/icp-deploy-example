mod utils;
mod deploy;

use clap::{Command, AppSettings, Arg};

pub const BUILD_PATH: &str = "target/wasm32-unknown-unknown/release";

// cargo run -p scripts -- --canister backend
#[tokio::main]
async fn main() -> Result<(), String> {

    let matches = Command::new("script")
		.setting(AppSettings::AllowExternalSubcommands)
        .setting(AppSettings::AllowLeadingHyphen)
        .arg(Arg::with_name("canister").long("canister").takes_value(true))
        .arg(Arg::with_name("network").long("network").takes_value(true))
        .get_matches();

    let default_network = "local".to_owned();
    let network = matches.get_one::<String>("network").unwrap_or(&default_network);

    let canister_name = &matches.get_one::<String>("canister").unwrap();

    crate::utils::build_wasm(&canister_name).await;
    // crate::utils::optimize_wasm(&canister_name).await;
    crate::deploy::deploy_wasm(network, canister_name).await;

    Ok(())
}
