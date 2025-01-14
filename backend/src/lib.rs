mod eth_call;
mod store_transactions;

// use candid::Principal;
use eth_call::{call_smart_contract, get_ecdsa_public_key};
use store_transactions::{store_transaction_hash, get_transaction_hashes};
use ethers_core::{abi::Address, k256::elliptic_curve::{sec1::ToEncodedPoint, PublicKey},  utils::keccak256};
use k256::Secp256k1;

const CONTRACT_ADDRESS: &str = "0xAed5d7b083ad30ad6B50f698427aD4907845AAc3";

const ABI_JSON: &str = r#"
   [
        {
            "inputs": [],
            "stateMutability": "nonpayable",
            "type": "constructor"
        },
        {
            "inputs": [],
            "name": "decreaseCount",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "getCount",
            "outputs": [
                {
                "internalType": "uint256",
                "name": "",
                "type": "uint256"
                }
            ],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "increaseCount",
            "outputs": [],
            "stateMutability": "nonpayable",
            "type": "function"
        }
    ]
"#;

fn get_abi() -> ethers_core::abi::Contract {
    serde_json::from_str::<ethers_core::abi::Contract>(ABI_JSON)
        .expect("Failed to parse ABI")
}

#[ic_cdk::update]
async fn call_increase_count() -> Result<String, String> {
    let abi = get_abi();
    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "increaseCount",
        &[],
        true,
        None,
    ).await?;

    let tx_hash = result.get(0)
        .ok_or("Expected transaction hash")?
        .clone()
        .into_string()
        .ok_or("Expected string value")?;

    store_transaction_hash(tx_hash.clone());
    Ok(format!("Increased count. Transaction hash: {}", tx_hash))
}

#[ic_cdk::update]
async fn get_count() -> Result<u64, String> {
    let abi = get_abi();
    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "getCount",
        &[],
        false,
        None,
    ).await?;

    let count = result.get(0)
        .ok_or("Expected count value")?
        .clone()
        .into_uint()
        .ok_or("Expected uint value")?;

    Ok(count.low_u64())
}

#[ic_cdk::update]
async fn call_decrease_count() -> Result<String, String> {
    let abi = get_abi();
    let result = call_smart_contract(
        CONTRACT_ADDRESS.to_string(),
        &abi,
        "decreaseCount",
        &[],
        true,
        None,
    ).await?;

    let tx_hash = result.get(0)
        .ok_or("Expected transaction hash")?
        .clone()
        .into_string()
        .ok_or("Expected string value")?;

    store_transaction_hash(tx_hash.clone());
    Ok(format!("Decreased count. Transaction hash: {}", tx_hash))
}

// Generating an ethereum address for the canister
#[ic_cdk::update] 
pub async fn get_canister_eth_address() -> String {
    let res = get_ecdsa_public_key().await; 
    let pubkey = res.public_key; 

    let key: PublicKey<Secp256k1> = PublicKey::from_sec1_bytes(&pubkey)
        .expect("Failed to pass the public key as SEC1"); 
    let point = key.to_encoded_point(false); 
    let point_bytes = point.as_bytes(); 
    assert_eq!(point_bytes[0], 0x04); 
    let hash = keccak256(&point_bytes[1..]); 
    let self_address = ethers_core::utils::to_checksum(&Address::from_slice(&hash[12..32]), None); 

    self_address
}

#[ic_cdk::update]
async fn get_stored_transaction_hashes() -> Vec<String> {
    get_transaction_hashes()
}

ic_cdk::export_candid!();
