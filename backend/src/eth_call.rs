use candid::Principal;
use ic_evm_utils::{eth_call::eth_call, eth_send_raw_transaction::contract_interaction, fees::estimate_transaction_fees};
use ic_evm_utils::eth_send_raw_transaction::ContractDetails; 
use evm_rpc_canister_types::{RpcApi, RpcServices}; 
use ethers_core::{
    abi::{Contract, Token},
    types::U256,
};
use evm_rpc_canister_types::{RpcService, EvmRpcCanister};
use ic_cdk::api::management_canister::ecdsa::{EcdsaKeyId, EcdsaCurve};

pub const EVM_RPC_CANISTER_ID: Principal = Principal::from_slice(b"\x00\x00\x00\x00\x02\x30\x00\xCC\x01\x01");
pub const EVM_RPC: EvmRpcCanister = EvmRpcCanister(EVM_RPC_CANISTER_ID);

// Modified to return a single reliable RPC service
fn get_rpc_services() -> RpcServices {
    RpcServices::Custom {
        chainId: 11155111, // Sepolia chain ID
        services: vec![
            RpcApi {
                url: "https://eth-sepolia.g.alchemy.com/v2/DZ4mML30eplCsoK1DGPPbhX5YfvR7ZhL".to_string(),
                headers: None,
            }
        ]
    }
}

pub async fn call_smart_contract(
    contract_address: String,
    abi: &Contract,
    function_name: &str,
    args: &[Token],
    is_write_operation: bool,
    value: Option<U256>,
) -> Result<Vec<Token>, String> {
    let contract_details = ContractDetails {
        contract_address,
        abi,
        function_name,
        args,
    };

    if is_write_operation {
        let fee_estimates = estimate_transaction_fees(
            10,
            get_rpc_services(),
            EVM_RPC,
        ).await;
        // ic_cdk::println!("Fee estimates: {:?}", fee_estimates);

        let result = contract_interaction(
            contract_details,
            value,
            get_rpc_services(),
            fee_estimates.max_priority_fee_per_gas,
            key_id(),
            vec![],
            EVM_RPC,
        ).await.map_err(|(code, msg)| format!("Error {:?}: {}", code, msg))?;
        ic_cdk::println!("Result from contract interaction: {:?}", result);

        Ok(vec![Token::String(result)])
    } else {
        let tokens = eth_call(
            contract_details,
            "latest",
            RpcService::Custom(RpcApi {
                url: "https://eth-sepolia.g.alchemy.com/v2/DZ4mML30eplCsoK1DGPPbhX5YfvR7ZhL".to_string(),
                headers: None,
            }),
            8192,
            EVM_RPC,
        ).await;
        
        Ok(tokens)
    }
}
// ECDSA key for your deployment environment:
// - For local dfx: "dfx_test_key"
// - For testnet: "test_key_1"
// - For mainnet: Use EcdsaKeyId::default()

fn key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: "test_key_1".to_string(),
    }
}