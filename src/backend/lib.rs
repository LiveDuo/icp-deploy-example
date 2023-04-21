
// dfx canister call backend len
#[ic_cdk_macros::query]
fn len() -> usize {
    let data = include_bytes!("../../data/dog-large.jpg");
    data.len()
}

