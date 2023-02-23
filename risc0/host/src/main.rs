// TODO: Update the name of the method loaded by the prover. E.g., if the method is `multiply`, replace `METHOD_NAME_ID` with `MULTIPLY_ID` and replace `METHOD_NAME_PATH` with `MULTIPLY_PATH`
use methods::{METHOD_NAME_ELF, METHOD_NAME_ID};
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::Prover;

fn main() {
    let source_list = vec![String::from("0x0000000000000000000000000000000000000000")];
    let block_list = vec![String::from("0x0000000000000000000000000000000000000001")];

    // Make the prover.
    let mut prover = Prover::new(METHOD_NAME_ELF, METHOD_NAME_ID).expect(
        "Prover should be constructed from valid method source code and corresponding method ID",
    );

    // TODO: Implement communication with the guest here
    // let source_list = to_vec(&source_list).unwrap();
    prover.add_input_u32_slice(&to_vec(&source_list).unwrap());
    prover.add_input_u32_slice(&to_vec(&block_list).unwrap());

    // Run prover & generate receipt
    let receipt = prover.run()
        .expect("Code should be provable unless it 1) had an error or 2) overflowed the cycle limit. See `embed_methods_with_options` for information on adjusting maximum cycle count.");

    // Optional: Verify receipt to confirm that recipients will also be able to verify your receipt
    receipt.verify(METHOD_NAME_ID).expect(
        "Code you have proven should successfully verify; did you specify the correct method ID?",
    );

    // TODO: Implement code for transmitting or serializing the receipt for other parties to verify here
}
