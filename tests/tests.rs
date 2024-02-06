extern crate schnorr_canister;

use secp256k1::{schnorr::Signature, Message, PublicKey};
use bitcoin_hashes::{Hash, sha256};

use candid::{decode_one, encode_one, CandidType,Principal};
use pocket_ic::{PocketIc, WasmResult};
use schnorr_canister::{SchnorrKeyIds, SchnorrPublicKey, SchnorrPublicKeyReply, SignWithSchnorr, SignWithSchnorrReply};
use serde::Deserialize;
use std::path::Path;


#[test]
fn test_sign_with_schnorr() {
    let pic = PocketIc::new();

    let my_principal = Principal::anonymous();
    // Create an empty canister as the anonymous principal and add cycles.
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 2_000_000_000_000);

    let wasm_bytes = load_schnorr_canister_wasm();
    pic.install_canister(canister_id, wasm_bytes, vec![], None);

    // Make sure the canister is properly initialized
    fast_forward(&pic, 5);
    
    let derivation_path = vec![vec![1u8; 4]]; // Example derivation path for signing
    let key_id = SchnorrKeyIds::TestKey1.to_key_id();
    let message = b"Test message";

    let digest = sha256::Hash::hash(message).to_byte_array();

    let payload: SignWithSchnorr = SignWithSchnorr {
        message: digest.to_vec(),
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let res: Result<SignWithSchnorrReply, String> = update(&pic, my_principal,  canister_id, "sign_with_schnorr", encode_one(&payload).unwrap());

    let sig = res.unwrap().signature;

    let payload = SchnorrPublicKey {
        canister_id: None,
        derivation_path: derivation_path.clone(),
        key_id: key_id.clone(),
    };

    let res: Result<SchnorrPublicKeyReply, String> =  update(&pic, my_principal,  canister_id, "schnorr_public_key", encode_one(&payload).unwrap());

    let pub_key_sec1 = res.unwrap().public_key;

    let pub_key = PublicKey::from_slice(&pub_key_sec1).unwrap().into();

    let sig = Signature::from_slice(&sig).unwrap();

    let msg = Message::from_digest_slice(&digest).unwrap();

    sig.verify(&msg, &pub_key).unwrap();
    
}

fn load_schnorr_canister_wasm() -> Vec<u8> {
    let wasm_path = Path::new("./target/wasm32-unknown-unknown/release/schnorr_canister.wasm.gz");

    std::fs::read(wasm_path).unwrap()
}

pub fn update<T: CandidType + for<'de> Deserialize<'de>>(
    ic: &PocketIc,
    sender: Principal,
    receiver: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, String> {
    match ic.update_call(receiver, sender, method, args) {
        Ok(WasmResult::Reply(data)) => Ok(decode_one(&data).unwrap()),
        Ok(WasmResult::Reject(error_message)) => Err(error_message.to_string()),
        Err(user_error) => Err(user_error.to_string()),
    }
}

pub fn fast_forward(ic: &PocketIc, ticks: u64) {
    for _ in 0..ticks-1 {
       ic.tick();
    }
}
