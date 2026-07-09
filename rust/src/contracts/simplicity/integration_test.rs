//! End-to-end P2PK covenant flow (offline, no regtest node required).

#[cfg(test)]
mod p2pk_e2e {
    use lwk_wollet::elements::pset::serialize::Deserialize;
    use lwk_wollet::elements::Transaction;

    use crate::contracts::blockdata::{ElementsOutPoint, ElementsTxOut, ElementsTxOutSecrets};
    use crate::contracts::external_utxo::ExternalUtxo;
    use crate::contracts::pset::{PsetBuilder, PsetInputBuilder, PsetOutputBuilder};
    use crate::contracts::signer::Signer;
    use crate::contracts::simplicity::{
        simplicity_control_block, simplicity_derive_xonly_pubkey, SimplicityArguments,
        SimplicityLogLevel, SimplicityProgram, SimplicityTypedValue, SimplicityWitnessValues,
        StateTaprootBuilder,
    };
    use crate::api::types::LiquidNetwork;

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

    #[test]
    fn test_p2pk_finalize_witness_shape() {
        let network = LiquidNetwork::Testnet;
        let signer = Signer::new(MNEMONIC.to_string(), network).unwrap();
        let derivation_path = "m/86'/1'/0'/0/0".to_string();
        let internal_key = simplicity_derive_xonly_pubkey(&signer, derivation_path.clone()).unwrap();
        let pubkey_bytes = internal_key.to_bytes();

        let program = SimplicityProgram::load_with_arguments(
            P2PK_SOURCE.to_string(),
            &SimplicityArguments::new().add_value(
                "ALICE_PUBLIC_KEY".into(),
                SimplicityTypedValue::u256(pubkey_bytes).unwrap(),
            ),
        )
        .unwrap();

        let spend = StateTaprootBuilder::new()
            .add_simplicity_leaf(0, &program.cmr())
            .unwrap()
            .finalize(&internal_key)
            .unwrap();
        let covenant_script = spend.script_pubkey();

        let funded_sats = 100_000u64;
        let send_amount = 50_000u64;
        let funding_out =
            ElementsTxOut::from_explicit(covenant_script.to_hex(), POLICY_ASSET.into(), funded_sats)
                .unwrap();

        let funding_txid = "ab".repeat(32);
        let outpoint = ElementsOutPoint::from_parts(funding_txid.clone(), 0).unwrap();

        let recipient = PsetOutputBuilder::new_op_return(
            vec![0x01],
            send_amount,
            POLICY_ASSET.into(),
        )
        .unwrap()
        .build()
        .unwrap();
        let change = PsetOutputBuilder::new_explicit(&covenant_script, funded_sats - send_amount, POLICY_ASSET.into())
            .unwrap()
            .build()
            .unwrap();

        let input_builder = PsetInputBuilder::from_prevout(&outpoint);
        input_builder.witness_utxo(&funding_out).unwrap();
        let input = input_builder.build().unwrap();

        let builder = PsetBuilder::new_v2();
        builder.add_input(&input).unwrap();
        builder.add_output(&recipient).unwrap();
        builder.add_output(&change).unwrap();
        let tx_bytes = builder.build().unwrap().extract_tx_bytes().unwrap();

        let utxos_hex = vec![hex::encode(funding_out.to_bytes())];
        let program_pk = internal_key.to_string_repr();

        let signature = program
            .create_p2pk_signature(
                &signer,
                derivation_path.clone(),
                tx_bytes.clone(),
                utxos_hex.clone(),
                0,
                network,
            )
            .unwrap();

        let witness_values = SimplicityWitnessValues::new().add_value(
            "ALICE_SIGNATURE".into(),
            SimplicityTypedValue::byte_array(signature),
        );

        let finalized = program
            .finalize_transaction_with_values(
                tx_bytes.clone(),
                program_pk.clone(),
                utxos_hex.clone(),
                0,
                &witness_values,
                network,
                SimplicityLogLevel::None,
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
                SimplicityLogLevel::None,
            )
            .unwrap();

        let control_block = simplicity_control_block(&run_result.cmr(), &internal_key).unwrap();
        let manual_witness = vec![
            run_result.witness_bytes(),
            run_result.program_bytes(),
            run_result.cmr().to_bytes(),
            control_block.to_bytes(),
        ];
        assert_eq!(*script_witness, manual_witness);
    }

    #[test]
    fn test_external_utxo_construction() {
        let network = LiquidNetwork::Testnet;
        let signer = Signer::new(MNEMONIC.to_string(), network).unwrap();
        let internal_key =
            simplicity_derive_xonly_pubkey(&signer, "m/86'/1'/0'/0/0".into()).unwrap();
        let pubkey_bytes = internal_key.to_bytes();

        let program = SimplicityProgram::load_with_arguments(
            P2PK_SOURCE.to_string(),
            &SimplicityArguments::new().add_value(
                "ALICE_PUBLIC_KEY".into(),
                SimplicityTypedValue::u256(pubkey_bytes).unwrap(),
            ),
        )
        .unwrap();

        let spend = StateTaprootBuilder::new()
            .add_simplicity_leaf(0, &program.cmr())
            .unwrap()
            .finalize(&internal_key)
            .unwrap();

        let txout = ElementsTxOut::from_explicit(
            spend.script_pubkey().to_hex(),
            POLICY_ASSET.into(),
            100_000,
        )
        .unwrap();
        let outpoint =
            ElementsOutPoint::from_parts("cd".repeat(32), 0).unwrap();
        let unblinded = ElementsTxOutSecrets::from_explicit(POLICY_ASSET.into(), 100_000).unwrap();

        let external = ExternalUtxo::from_unchecked_data(
            &outpoint,
            &txout,
            &unblinded,
            700,
        );
        assert_eq!(external.inner.max_weight_to_satisfy, 700);
    }
}
