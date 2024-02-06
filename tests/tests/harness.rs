use fuels::{
    accounts::predicate::Predicate,
    prelude::*,
    tx::Bytes32,
    tx::StorageSlot,
    types::{Bits256, Identity, ContractId},
};
use rand::prelude::{Rng, SeedableRng, StdRng};
use std::fs;

// Load abi from json
abigen!(
    Contract(
        name = "SimpleContract",
        abi = "../simple_contract/out/debug/simple_contract-abi.json"
    ),
    Script(name = "VerifierScript", abi = "../bytecode_verifier_script/out/debug/verifier_script-abi.json"),
    Contract(
        name = "SwapperContract",
        abi = "../configurable_swapper_contract/out/debug/swapper_contract-abi.json"
    ),    
    Predicate(name = "SimplePredicate", abi = "../simple_predicate/out/debug/simple_predicate-abi.json"),
);

mod contract {

    use super::*;

    /// Test to make sure swapping configurables in a contract works
    #[tokio::test]
    async fn can_swap_configurables() {
        let (swapper_instance, _id, wallet) = get_swapper_contract_instance().await;

        // Get the bytecode for the contract
        let bytecode_filepath = "../simple_contract/out/debug/simple_contract.bin";
        let file_bytecode = fs::read(bytecode_filepath).unwrap();

        // Build the configurable changes
        let offset = 68;
        let my_configurables = build_configurables(offset);

        // Call the contract to swap the configurables
        let result_bytecode = swapper_instance
            .methods()
            .swap_configurable(file_bytecode.clone(), my_configurables.clone())
            .call()
            .await
            .unwrap()
            .value;

        // Verify the bytecode has the correct changes made for the configurables
        assert!(file_bytecode != result_bytecode);
        for iter in 0..7 {
            assert_eq!(
                result_bytecode
                    .get(my_configurables.get(0).unwrap().0 as usize + iter)
                    .unwrap(),
                my_configurables.get(0).unwrap().1.get(iter).unwrap()
            );
        }

        // Deploy the new simple contract with the bytecode that contains the changes
        let rng = &mut StdRng::seed_from_u64(2322u64);
        let salt: [u8; 32] = rng.gen();
        let storage_vec = Vec::<StorageSlot>::new();
        let result_id = Contract::new(result_bytecode, salt.into(), storage_vec)
            .deploy(&wallet, TxPolicies::default())
            .await
            .unwrap();
        let result_instance = SimpleContract::new(result_id.clone(), wallet.clone());

        // Assert that we get the exected value for the configurable from the deployed simple contract.
        let result_u64 = result_instance
            .methods()
            .test_function()
            .call()
            .await
            .unwrap()
            .value;
        assert!(result_u64 == 119);
    }

    /// Test to make sure computing the bytecode root of a contract works
    #[tokio::test]
    async fn can_compute_bytecode_root() {
        let (simple_instance, id, wallet, root) = get_simple_contract_instance().await;

        // Get the bytecode of simple contract contract and the configuables
        let bytecode_filepath = "../simple_contract/out/debug/simple_contract.bin";
        let file_bytecode = fs::read(bytecode_filepath).unwrap();
        let offset = 68;
        let my_configurables = build_configurables(offset);

        // Create verifier script instance
        let script_bin_path = "../bytecode_verifier_script/out/debug/verifier_script.bin";
        let script_instance = VerifierScript::new(wallet, script_bin_path);

        // Run the script to verify the expected bytecode root
        let deployed_contract_id = Identity::ContractId(id);
        let result = script_instance
            .main(deployed_contract_id, file_bytecode.clone(), my_configurables)
            .with_contracts(&[&simple_instance])
            .call()
            .await
            .unwrap()
            .value;

        assert_eq!(result, Bits256(*root));
    }

    /// Helper function to deploy the simple contract 
    pub async fn get_simple_contract_instance() -> (
        SimpleContract<WalletUnlocked>,
        ContractId,
        WalletUnlocked,
        Bytes32,
    ) {
        // Launch a local network and deploy the contract
        let mut wallets = launch_custom_provider_and_get_wallets(
            WalletsConfig::new(
                Some(1),             /* Single wallet */
                Some(1),             /* Single coin (UTXO) */
                Some(1_000_000_000), /* Amount per coin */
            ),
            None,
            None,
        )
        .await
        .unwrap();
        let wallet = wallets.pop().unwrap();

        let configurables = SimpleContractConfigurables::new().with_VALUE(119);

        let id = Contract::load_from(
            "../simple_contract/out/debug/simple_contract.bin",
            LoadConfiguration::default().with_configurables(configurables.clone()),
        )
        .unwrap()
        .deploy(&wallet, TxPolicies::default())
        .await
        .unwrap();

        let instance = SimpleContract::new(id.clone(), wallet.clone());

        // Fetch the bytecode root
        let root = Contract::load_from(
            "../simple_contract/out/debug/simple_contract.bin",
            LoadConfiguration::default().with_configurables(configurables),
        )
        .unwrap()
        .code_root();

        (instance, id.into(), wallet, root)
    }
}

mod predicate {
    use super::*;

    /// Test to make sure swapping configurables in a predicate works
    #[tokio::test]
    async fn can_swap_configurables() {
        let (swapper_instance, _id, wallet) = get_swapper_contract_instance().await;

        // Get the bytecode for the predicate
        let bytecode_filepath = "../simple_predicate/out/debug/simple_predicate.bin";
        let file_bytecode = fs::read(bytecode_filepath).unwrap();

        // Build the configurable changes
        let offset = 188;
        let my_configurables = build_configurables(offset);

        // Call the contract to swap the configurables
        let result_bytecode = swapper_instance
            .methods()
            .swap_configurable(file_bytecode.clone(), my_configurables.clone())
            .call()
            .await
            .unwrap()
            .value;

        // Verify the bytecode has the correct changes made for the configurables
        assert!(file_bytecode != result_bytecode);
        for iter in 0..7 {
            assert_eq!(
                result_bytecode
                    .get(my_configurables.get(0).unwrap().0 as usize + iter)
                    .unwrap(),
                my_configurables.get(0).unwrap().1.get(iter).unwrap()
            );
        }

        // Create the new simple predicate with the bytecode that contains the changes
        let provider = wallet.try_provider().unwrap();
        let predicate_data = SimplePredicateEncoder::encode_data(119);
        let result_instance = Predicate::from_code(result_bytecode)
            .with_provider(provider.clone())
            .with_data(predicate_data);

        // Fund predicate
        let amount_to_predicate = 512;
        wallet
            .transfer(
                result_instance.address(),
                amount_to_predicate,
                BASE_ASSET_ID,
                TxPolicies::default(),
            )
            .await
            .unwrap();
        let predicate_balance = result_instance.get_asset_balance(&BASE_ASSET_ID).await.unwrap();
        assert_eq!(predicate_balance, amount_to_predicate);

        // Assert that we can spend the predicate with the exected value for the configurable.
        result_instance
            .transfer(
                wallet.address(),
                1,
                BASE_ASSET_ID,
                TxPolicies::default(),
            )
            .await
            .unwrap();
    }

    /// Test to make sure computing the address of a predicate works
    #[tokio::test]
    async fn can_compute_address() {
        let wallet = get_wallet().await;

        // Get the bytecode of simple predicate and the configuables
        let bytecode_filepath = "../simple_predicate/out/debug/simple_predicate.bin";
        let file_bytecode = fs::read(bytecode_filepath).unwrap();

        // Build the configurable changes
        let offset = 188;
        let my_configurables = build_configurables(offset);

        // Create verifier script instance
        let script_bin_path = "../bytecode_verifier_script/out/debug/verifier_script.bin";
        let script_instance = VerifierScript::new(wallet.clone(), script_bin_path);

        // Create preciate
        let provider = wallet.try_provider().unwrap();
        let predicate_data = SimplePredicateEncoder::encode_data(119);
        let configurables = SimplePredicateConfigurables::new().with_VALUE(119);
        let predicate_instance = Predicate::load_from(bytecode_filepath)
            .unwrap()
            .with_configurables(configurables)
            .with_provider(provider.clone())
            .with_data(predicate_data);
        let prediate_address = Identity::Address(predicate_instance.address().into());

        // Run the script to verify the expected bytecode root
        let result = script_instance
            .main(prediate_address, file_bytecode, my_configurables)
            .call()
            .await
            .unwrap()
            .value;

        assert_eq!(result, Bits256(*predicate_instance.address().hash()));
    }
}

/// Helper function to generate a wallet
pub(crate) async fn get_wallet() -> WalletUnlocked {
    // Launch a local network and deploy the contract
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await
    .unwrap();

    wallets.pop().unwrap()
}

/// Helper function to deploy the swapper contract
pub(crate) async fn get_swapper_contract_instance() -> (
    SwapperContract<WalletUnlocked>,
    ContractId,
    WalletUnlocked,
) {
    let base_asset = AssetConfig {
        id: BASE_ASSET_ID,
        num_coins: 1,
        coin_amount: 100_000,
    };

    let wallet_config = WalletsConfig::new_multiple_assets(1, vec![base_asset]);
    let wallet = launch_custom_provider_and_get_wallets(wallet_config, None, None)
        .await
        .unwrap()
        .pop()
        .unwrap();

    let id = Contract::load_from(
        "../configurable_swapper_contract/out/debug/swapper_contract.bin",
        LoadConfiguration::default(),
    )
    .unwrap()
    .deploy(&wallet, TxPolicies::default())
    .await
    .unwrap();

    let instance = SwapperContract::new(id.clone(), wallet.clone());

    (instance, id.into(), wallet)
}

/// Helper function to generate the configurable changes needed. Hardcoded 119u64 for now
pub fn build_configurables(offset: u64) -> Vec<(u64, Vec<u8>)> {
    // Build the configurable changes from the abi.json
    // This is hardcoded for now. From the json below we know it's at offset 68 for simple_contract

    // "configurables": [
    // {
    //     "name": "VALUE",
    //     "configurableType": {
    //       "name": "",
    //       "type": 0,
    //       "typeArguments": null
    //     },
    //     "offset": 68
    //   }
    // ]

    let mut my_configurables: Vec<(u64, Vec<u8>)> = Vec::new();
    let mut data: Vec<u8> = Vec::new();

    data.push(0u8);
    data.push(0u8);
    data.push(0u8);
    data.push(0u8);
    data.push(0u8);
    data.push(0u8);
    data.push(0u8);
    data.push(119u8);
    my_configurables.push((offset, data));

    my_configurables
}
