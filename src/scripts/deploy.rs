
use candid::{Decode, Encode, Principal};
use ic_agent::{identity::*};

use sha2::{Sha256, Digest};

use std::process::Command;
use std::path::Path;

const CHUNK_SIZE: usize = 2_000_000;

pub async fn deploy_wasm(network: &str, canister: &str) {
    
    // get deployer principal
    let mut canister_opt = crate::utils::get_canister_id("deployer", Some(&network)).await;
    if canister_opt.is_none() {
        Command::new("dfx").arg("deploy").arg("deployer").spawn().unwrap().wait().unwrap();
        canister_opt = crate::utils::get_canister_id("deployer", Some(&network)).await;
    }
    let deployer_principal = Principal::from_text(&canister_opt.unwrap()).unwrap();

    // get ic agent
    let host = crate::utils::get_host(network).await;
    let agent_url = "http://".to_string() + &host;
    let pem_file = std::env::var("HOME").unwrap() + "/.config/dfx/identity/default/identity.pem";
    let identity = BasicIdentity::from_pem_file(&pem_file).unwrap();
    let agent = crate::utils::get_agent(&agent_url, identity).await;
    
    // build wasm
    crate::utils::build_wasm(&canister).await;
    // crate::utils::optimize_wasm(&canister).await;

    // check wasm hash
    let build_path = Path::join(Path::new(crate::BUILD_PATH), canister.to_string() + ".wasm");
    let content = std::fs::read(build_path).unwrap();
    let mut prepare_query = agent.query(&deployer_principal, "get_wasm_hash");
    let response = prepare_query.with_arg(Encode!().unwrap()).call().await.unwrap();
    let wasm_hash = Decode!(&response, Vec<u8>).unwrap();    
    let mut hasher = Sha256::new();
    hasher.update(content.clone());
    if hasher.finalize()[..].to_vec() == wasm_hash { println!("Already deployed."); std::process::exit(0); }

    // upload chunk
    let chunks: Vec<Vec<u8>> = content.chunks(CHUNK_SIZE).map(|s| s.into()).collect();
    for chunk in chunks {
        let mut prepare_update = agent.update(&deployer_principal, "append_chunk");
        let waiter = garcon::Delay::builder().build();
        let response = prepare_update.with_arg(Encode!(&chunk).unwrap()).call_and_wait(waiter).await.unwrap();
        let _ = Decode!(&response, Result<(), String>).unwrap();
    }

    // deploy wasm
    let mut prepare_update = agent.update(&deployer_principal, "deploy_canister");
    let waiter = garcon::Delay::builder().build();
    let response = prepare_update.with_arg(Encode!().unwrap()).call_and_wait(waiter).await.unwrap();
    let principal = Decode!(&response, Result<Principal, String>).unwrap();
    
    // update canister ids
    crate::utils::update_canister_id("backend", &principal.clone().unwrap().to_text(), Some(network)).await.unwrap();
    println!("Principal: {}", principal.unwrap());

}

