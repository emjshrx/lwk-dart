//! End-to-end P2PK covenant flow (offline, no regtest node required).

#[cfg(test)]
mod p2pk_e2e {
    use std::collections::HashMap;
    use std::str::FromStr;

    use lwk_simplicity::simplicityhl::num::U256;
    use lwk_simplicity::simplicityhl::parse::ParseFromStr;
    use lwk_simplicity::simplicityhl::str::WitnessName;
    use lwk_simplicity::simplicityhl::value::ValueConstructible;
    use lwk_simplicity::simplicityhl::{Arguments, Value, WitnessValues};
    use lwk_wollet::elements::bitcoin::XOnlyPublicKey;
    use lwk_wollet::elements::confidential::{
        Asset, AssetBlindingFactor, Nonce, Value as ConfValue, ValueBlindingFactor,
    };
    use lwk_wollet::elements::encode::serialize;
    use lwk_wollet::elements::pset::serialize::Deserialize;
    use lwk_wollet::elements::pset::{Input, Output, PartiallySignedTransaction};
    use lwk_wollet::elements::{
        AssetId, OutPoint, Script, Transaction, TxOut, TxOutSecrets, TxOutWitness, Txid,
    };

    use crate::api::types::LiquidNetwork;
    use crate::contracts::simplicity::{
        derive_keypair, simplicity_control_block, SimplicityProgram, StateTaprootBuilder,
    };

    const P2PK_SOURCE: &str = concat!(
        "fn main() {\n",
        "    jet::bip_0340_verify(",
        "(param::ALICE_PUBLIC_KEY, jet::sig_all_hash()), ",
        "witness::ALICE_SIGNATURE)\n",
        "}\n",
    );

    /// Liquid testnet L-BTC asset id.
    const POLICY_ASSET: &str =
        "144c654344aa71608c11a65921ccb7398bd9734057d5c98e002dd31308e14d29";

    const MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    fn explicit_txout(script: Script, asset_id: &str, satoshi: u64) -> TxOut {
        TxOut {
            script_pubkey: script,
            asset: Asset::Explicit(AssetId::from_str(asset_id).unwrap()),
            value: ConfValue::Explicit(satoshi),
            nonce: Nonce::Null,
            witness: TxOutWitness::default(),
        }
    }

    fn p2pk_args(pubkey: &XOnlyPublicKey) -> Arguments {
        let mut map = HashMap::new();
        map.insert(
            WitnessName::parse_from_str("ALICE_PUBLIC_KEY").unwrap(),
            Value::u256(U256::from_byte_array(pubkey.serialize())),
        );
        Arguments::from(map)
    }

    fn witness_signature(signature: Vec<u8>) -> WitnessValues {
        let mut map = HashMap::new();
        map.insert(
            WitnessName::parse_from_str("ALICE_SIGNATURE").unwrap(),
            Value::byte_array(signature),
        );
        WitnessValues::from(map)
    }

    #[test]
    fn test_p2pk_finalize_witness_shape() {
        let network = LiquidNetwork::Testnet;
        let derivation_path = "m/86'/1'/0'/0/0";
        let keypair = derive_keypair(MNEMONIC, network, derivation_path).unwrap();
        let internal_key = keypair.x_only_public_key().0;

        let program =
            SimplicityProgram::load_with_arguments(P2PK_SOURCE.to_string(), &p2pk_args(&internal_key))
                .unwrap();

        let spend = StateTaprootBuilder::new()
            .add_simplicity_leaf(0, program.cmr())
            .unwrap()
            .finalize(&internal_key)
            .unwrap();
        let covenant_script = spend.script_pubkey();

        let funded_sats = 100_000u64;
        let send_amount = 50_000u64;
        let funding_out = explicit_txout(covenant_script.clone(), POLICY_ASSET, funded_sats);

        let funding_txid = Txid::from_str(&"ab".repeat(32)).unwrap();
        let outpoint = OutPoint::new(funding_txid, 0);

        let mut input = Input::from_prevout(outpoint);
        input.witness_utxo = Some(funding_out.clone());

        let recipient = Output::new_explicit(
            Script::new_op_return(&[0x01]),
            send_amount,
            AssetId::from_str(POLICY_ASSET).unwrap(),
            None,
        );
        let change = Output::new_explicit(
            covenant_script,
            funded_sats - send_amount,
            AssetId::from_str(POLICY_ASSET).unwrap(),
            None,
        );

        let mut pset = PartiallySignedTransaction::new_v2();
        pset.add_input(input);
        pset.add_output(recipient);
        pset.add_output(change);
        let tx_bytes = serialize(&pset.extract_tx().unwrap());

        let utxos_hex = vec![hex::encode(serialize(&funding_out))];
        let program_pk = internal_key.to_string();

        let signature = program
            .create_p2pk_signature(&keypair, tx_bytes.clone(), utxos_hex.clone(), 0, network)
            .unwrap();
        let witness_values = witness_signature(signature);

        let finalized = program
            .finalize_transaction(
                tx_bytes.clone(),
                program_pk.clone(),
                utxos_hex.clone(),
                0,
                &witness_values,
                network,
            )
            .unwrap();

        let tx = Transaction::deserialize(&finalized).unwrap();
        let script_witness = &tx.input[0].witness.script_witness;
        assert_eq!(
            script_witness.len(),
            4,
            "Simplicity witness stack must have 4 elements"
        );

        let run_result = program
            .run(
                tx_bytes,
                program_pk,
                utxos_hex,
                0,
                &witness_values,
                network,
            )
            .unwrap();

        let control_block = simplicity_control_block(run_result.cmr(), &internal_key).unwrap();
        let manual_witness = vec![
            run_result.witness_bytes(),
            run_result.program_bytes(),
            run_result.cmr().to_byte_array().to_vec(),
            control_block.serialize(),
        ];
        assert_eq!(*script_witness, manual_witness);
    }

    #[test]
    fn test_external_utxo_construction() {
        let network = LiquidNetwork::Testnet;
        let keypair = derive_keypair(MNEMONIC, network, "m/86'/1'/0'/0/0").unwrap();
        let internal_key = keypair.x_only_public_key().0;

        let program =
            SimplicityProgram::load_with_arguments(P2PK_SOURCE.to_string(), &p2pk_args(&internal_key))
                .unwrap();

        let spend = StateTaprootBuilder::new()
            .add_simplicity_leaf(0, program.cmr())
            .unwrap()
            .finalize(&internal_key)
            .unwrap();

        let txout = explicit_txout(spend.script_pubkey(), POLICY_ASSET, 100_000);
        let outpoint = OutPoint::new(Txid::from_str(&"cd".repeat(32)).unwrap(), 0);
        let asset = AssetId::from_str(POLICY_ASSET).unwrap();
        let unblinded = TxOutSecrets::new(
            asset,
            AssetBlindingFactor::zero(),
            100_000,
            ValueBlindingFactor::zero(),
        );

        let external = lwk_wollet::ExternalUtxo {
            outpoint,
            txout,
            tx: None,
            unblinded,
            max_weight_to_satisfy: 700,
        };
        assert_eq!(external.max_weight_to_satisfy, 700);
    }
}
