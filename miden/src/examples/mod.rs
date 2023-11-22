use std::{io::Write, time::Instant};

use clap::Parser;
use miden::{ExecutionProof, Host, Program, ProgramInfo, ProvingOptions, StackInputs};
use processor::{ExecutionOptions, ExecutionOptionsError, ONE, ZERO};

pub mod fibonacci;

// EXAMPLE
// ================================================================================================

pub struct Example<H>
where
    H: Host,
{
    pub program: Program,
    pub stack_inputs: StackInputs,
    pub host: H,
    pub num_outputs: usize,
    pub expected_result: Vec<u64>,
}

// EXAMPLE OPTIONS
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[clap(about = "Run an example miden program")]
pub struct ExampleOptions {
    #[clap(subcommand)]
    pub example: ExampleType,

    /// Number of cycles the program is expected to consume
    #[clap(short = 'e', long = "exp-cycles", default_value = "64")]
    expected_cycles: u32,

    /// Maximum number of cycles a program is allowed to consume
    #[clap(short = 'm', long = "max-cycles", default_value = "4294967295")]
    max_cycles: u32,

    /// Enable generation of proofs suitable for recursive verification
    #[clap(short = 'r', long = "recursive")]
    recursive: bool,

    /// Security level for execution proofs generated by the VM
    #[clap(short = 's', long = "security", default_value = "96bits")]
    security: String,
}

#[derive(Debug, Clone, Parser)]
//#[clap(about = "available examples")]
pub enum ExampleType {
    /// Compute a Fibonacci sequence of the specified length
    Fib {
        /// Length of Fibonacci sequence
        #[clap(short = 'n', default_value = "1024")]
        sequence_length: usize,
    },
}

impl ExampleOptions {
    pub fn get_proof_options(&self) -> Result<ProvingOptions, ExecutionOptionsError> {
        let exec_options = ExecutionOptions::new(Some(self.max_cycles), self.expected_cycles)?;
        Ok(match self.security.as_str() {
            "96bits" => ProvingOptions::with_96_bit_security(self.recursive),
            "128bits" => ProvingOptions::with_128_bit_security(self.recursive),
            other => panic!("{} is not a valid security level", other),
        }
        .with_execution_options(exec_options))
    }

    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");

        // configure logging
        env_logger::Builder::new()
            .format(|buf, record| writeln!(buf, "{}", record.args()))
            .filter_level(log::LevelFilter::Debug)
            .init();

        let proof_options = self.get_proof_options().map_err(|err| format!("{err}"))?;

        // instantiate and prepare the example
        let example = match self.example {
            ExampleType::Fib { sequence_length } => fibonacci::get_example(sequence_length),
        };

        let Example {
            program,
            stack_inputs,
            host,
            num_outputs,
            expected_result,
            ..
        } = example;
        println!("--------------------------------");

        // execute the program and generate the proof of execution
        let now = Instant::now();
        let (stack_outputs, proof) =
            miden::prove(&program, stack_inputs.clone(), host, proof_options).unwrap();
        println!("--------------------------------");

        println!(
            "Executed program in {} ms",
            //hex::encode(program.hash()), // TODO: include into message
            now.elapsed().as_millis()
        );
        println!("Stack outputs: {:?}", stack_outputs.stack_truncated(num_outputs));
        assert_eq!(
            expected_result,
            stack_outputs.stack_truncated(num_outputs),
            "Program result was computed incorrectly"
        );

        // serialize the proof to see how big it is
        let proof_bytes = proof.to_bytes();
        println!("Execution proof size: {} KB", proof_bytes.len() / 1024);
        println!("Execution proof security: {} bits", proof.security_level());
        println!("--------------------------------");

        // verify that executing a program with a given hash and given inputs
        // results in the expected output
        let proof = ExecutionProof::from_bytes(&proof_bytes).unwrap();
        let now = Instant::now();
        let program_info = ProgramInfo::from(program);

        match miden::verify(program_info, stack_inputs, stack_outputs, proof) {
            Ok(_) => println!("Execution verified in {} ms", now.elapsed().as_millis()),
            Err(err) => println!("Failed to verify execution: {}", err),
        }

        Ok(())
    }
}

// TESTS
// ================================================================================================

#[cfg(test)]
pub fn test_example<H>(example: Example<H>, fail: bool)
where
    H: Host,
{
    let Example {
        program,
        stack_inputs,
        host,
        num_outputs,
        expected_result,
    } = example;

    let (mut outputs, proof) =
        miden::prove(&program, stack_inputs.clone(), host, ProvingOptions::default()).unwrap();

    assert_eq!(
        expected_result,
        outputs.stack_truncated(num_outputs),
        "Program result was computed incorrectly"
    );

    let kernel = miden::Kernel::default();
    let program_info = ProgramInfo::new(program.hash(), kernel);

    if fail {
        outputs.stack_mut()[0] += 1;
        assert!(miden::verify(program_info, stack_inputs, outputs, proof).is_err())
    } else {
        assert!(miden::verify(program_info, stack_inputs, outputs, proof).is_ok());
    }
}
