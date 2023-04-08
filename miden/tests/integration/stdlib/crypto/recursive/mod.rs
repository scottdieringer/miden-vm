use miden::{
    crypto::MerkleStore, Assembler, MemAdviceProvider, ProgramInfo,
    ProofOptions as MidenProofOptions,
};
use miden_air::{Felt, HashFunction, PublicInputs};

use winter_air::{FieldExtension, ProofOptions as WinterProofOptions};

mod verifier_recursive;

use verifier_recursive::VerifierError;

use crate::build_test;

#[test]
fn stark_verifier_e2f4() {
    // An example MASM program to be verified inside Miden VM
    // Note that output stack-overflow is not yet supported because of the way we handle public inputs
    // in the STARK verifier is not yet general enough. Thus the output stack should be of size exactly 16.
    let example_source = "begin
            repeat.32
                swap dup.1 add
            end
        end";
    let mut stack_inputs = vec![0_u64; 16];
    stack_inputs[15] = 0;
    stack_inputs[14] = 1;

    let (initial_stack, tape, store, advice_map) =
        generate_recursive_verifier_data(example_source, stack_inputs).unwrap();

    // Verify inside Miden VM
    let source = "
        use.std::crypto::stark::verifier

        begin
            exec.verifier::verify
        end
        ";

    let test = build_test!(source, &initial_stack, &tape, store, advice_map);

    test.expect_stack(&[]);
}

// Helper function for recursive verification
pub fn generate_recursive_verifier_data(
    source: &str,
    stack_inputs: Vec<u64>,
) -> Result<(Vec<u64>, Vec<u64>, MerkleStore, Vec<([u8; 32], Vec<Felt>)>), VerifierError> {
    let program = Assembler::default().compile(&source).unwrap();
    let stack_inputs = crate::helpers::StackInputs::try_from_values(stack_inputs).unwrap();
    let advice_inputs = crate::helpers::AdviceInputs::default();
    let advice_provider = MemAdviceProvider::from(advice_inputs);

    let options = WinterProofOptions::new(27, 8, 16, FieldExtension::Quadratic, 4, 7);
    let proof_options = MidenProofOptions {
        hash_fn: HashFunction::Rpo256,
        options,
    };
    let (stack_outputs, proof) =
        miden::prove(&program, stack_inputs.clone(), advice_provider, proof_options).unwrap();

    let program_info = ProgramInfo::from(program);

    // build public inputs and generate the advice data needed for recursive proof verification
    let pub_inputs = PublicInputs::new(program_info, stack_inputs, stack_outputs);
    let (_, proof) = proof.into_parts();
    Ok(verifier_recursive::generate_advice_inputs(proof, pub_inputs).unwrap())
}
