
use candid::{CandidType, Principal, Deserialize};

use std::cell::RefCell;

use ic_cdk::api::management_canister::main::*;

use sha2::{Sha256, Digest};

#[derive(Default, CandidType, Deserialize, Clone)]
pub struct State { wasm: Vec<u8>, canister_id: Option<Principal>, hash: Vec<u8> }

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[ic_cdk_macros::query]
pub async fn get_wasm_hash() -> Vec<u8> {
    STATE.with(|s| { s.borrow().hash.clone() })
}

#[ic_cdk_macros::update]
pub async fn append_chunk(chunk: Vec<u8>) -> Result<(), String> {
    ic_cdk::println!("Chunk Len: {:?}", chunk.len());

    STATE.with(|s| { s.borrow_mut().wasm.extend(chunk); });

    Ok(())
}

#[ic_cdk_macros::update]
pub async fn deploy_canister() -> Result<Principal, String> {

    let canister_id;

    // create canister
    let canister_id_opt = STATE.with(|s| { s.borrow().canister_id.clone() });
    if canister_id_opt.is_none() {
        let settings = CanisterSettings { controllers: Some(vec![ic_cdk::api::id()]), ..Default::default() };
        let create_args = CreateCanisterArgument { settings: Some(settings) };
        let create_result: (CanisterIdRecord,) = ic_cdk::api::call::call_with_payment(Principal::management_canister(), "create_canister", (create_args,), 200_000_000_000).await.unwrap();
        canister_id = create_result.0.canister_id;

        // save canister id
        STATE.with(|s| { s.borrow_mut().canister_id = Some(canister_id) });
        
    } else {
        canister_id = canister_id_opt.unwrap();
    };

    // install code
    let wasm_module = STATE.with(|s| { s.borrow().wasm.clone() });
    let mode = if canister_id_opt.is_none() { CanisterInstallMode::Install } else { CanisterInstallMode::Upgrade };
	let install_args = InstallCodeArgument { mode, canister_id, wasm_module: wasm_module.clone(), arg: vec![], };
	let _: ((), ) = ic_cdk::call(Principal::management_canister(), "install_code", (install_args,),).await.unwrap();

    // store hash
    let mut hasher = Sha256::new();
    hasher.update(wasm_module.clone());
    let hash = &hasher.finalize()[..];
    STATE.with(|s| { s.borrow_mut().hash = hash.to_vec(); });
    
    // cleanup wasm
    STATE.with(|s| { s.borrow_mut().wasm = vec![]; });

	Ok(canister_id)
}

#[ic_cdk_macros::pre_upgrade]
fn pre_upgrade() {
    let state = STATE.with(|s| s.borrow().to_owned());
    ic_cdk::storage::stable_save((state,)).unwrap();
}

#[ic_cdk_macros::post_upgrade]
fn post_upgrade() {
    let (s_prev,): (State,) = ic_cdk::storage::stable_restore().unwrap();
    STATE.with(|s|{ *s.borrow_mut() = s_prev; });
}
