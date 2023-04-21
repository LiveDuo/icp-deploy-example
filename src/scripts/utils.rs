
use std::path::{*};
use std::str;
use std::fs::{self, File};
use std::process::Command;

use ic_agent::{Agent, identity::*, agent::*};

pub fn get_project_root() -> Result<PathBuf, String> {
    let path = std::env::current_dir().unwrap();
    let mut path_ancestors = path.as_path().ancestors();
    while let Some(p) = path_ancestors.next() {
        let has_cargo = fs::read_dir(p)
                .unwrap()
                .into_iter()
                .any(|p| p.unwrap().file_name() == std::ffi::OsString::from("Cargo.lock"));
        if has_cargo {
            return Ok(PathBuf::from(p))
        }
    }
    Err("Not found".to_string())
}

pub async fn get_canister_id(name: &str, network_opt: Option<&str>) -> Option<String> {
    let network = network_opt.unwrap_or("local");
    let path = std::path::Path::join(&get_project_root().unwrap(), Path::new(".dfx").join(network).join("canister_ids.json"));
    let file_opt = File::open(path);
    if file_opt.is_err() { return None }

    let json: serde_json::Value = serde_json::from_reader(file_opt.unwrap()).unwrap();
    if let Some(networks) = json.get(name) {
        let principal = networks.get(network).unwrap().as_str().unwrap();
        return Some(principal.to_string())
    } else {
        None
    }
}

pub async fn update_canister_id(name: &str, principal: &str, network_opt: Option<&str>) -> Result<(), String> {
    let network = network_opt.unwrap_or("local");
    let path = std::path::Path::join(&get_project_root().unwrap(), Path::new(".dfx").join(network).join("canister_ids.json"));
    let file_opt = File::open(&path);
    if file_opt.is_err() { return Err("File not found".to_string()) }

    let json: serde_json::Value = serde_json::from_reader(file_opt.unwrap()).unwrap();
    if let serde_json::Value::Object(mut m) = json {
        m.insert(name.to_string(), serde_json::json!({network.to_string(): principal}));
        std::fs::write(&path, serde_json::to_string_pretty(&m).unwrap()).unwrap();
    } else {
        return Err("Invalid json".to_string())
    };
    Ok(())
}

pub async fn get_host(network: &str) -> String {
    let path = std::path::Path::join(&get_project_root().unwrap(), "dfx.json");
    let file = File::open(path).unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    if json.get("networks").is_none() { return "127.0.0.1:8000".to_string() };

    let networks = json.get("networks").unwrap();
    let config = networks.get(network).unwrap();
    let host = config.get("bind").unwrap().as_str().unwrap();
    return host.to_string()
}

pub async fn get_agent(url: &str, identity: impl Identity + 'static) -> Agent {
    let transport = http_transport::ReqwestHttpReplicaV2Transport::create(url).unwrap();
    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(identity)
        .build()
        .unwrap();

    agent.fetch_root_key().await.unwrap();
    return agent;
}

pub async fn build_wasm(canister: &str) {
    let mut build_process = Command::new("cargo")
        .arg("build")
        .arg("--target")
        .arg("wasm32-unknown-unknown")
        .arg("--package")
        .arg(canister)
        .arg("--release")
        .spawn()
        .unwrap();
    let _ = build_process.wait().unwrap();
}

#[allow(dead_code)]
pub async fn optimize_wasm(canister: &str) {
    let path = Path::join(Path::new(crate::BUILD_PATH), canister.to_string() + ".wasm");
    let mut optimize_process = Command::new("ic-cdk-optimizer")
        .arg(&path)
        .arg("-o")
        .arg(&path)
        .spawn()
        .unwrap();
    let _ = optimize_process.wait().unwrap();
}
