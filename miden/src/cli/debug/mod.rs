use super::data::{Debug, InputFile, Libraries, ProgramFile};
use clap::Parser;
use rustyline::{error::ReadlineError, Config, DefaultEditor, EditMode};
use std::path::PathBuf;

mod command;
use command::DebugCommand;

mod executor;
use executor::DebugExecutor;

#[derive(Debug, Clone, Parser)]
#[clap(about = "Debug a miden program")]
pub struct DebugCmd {
    /// Path to .masm assembly file
    #[clap(short = 'a', long = "assembly", value_parser)]
    assembly_file: PathBuf,
    /// Path to input file
    #[clap(short = 'i', long = "input", value_parser)]
    input_file: Option<PathBuf>,
    /// Enable vi edit mode
    #[clap(long = "vi", long = "vim_edit_mode")]
    vim_edit_mode: Option<String>,
    /// Paths to .masl library files
    #[clap(short = 'l', long = "libraries", value_parser)]
    library_paths: Vec<PathBuf>,
}

impl DebugCmd {
    pub fn execute(&self) -> Result<(), String> {
        log::info!("============================================================\n");
        log::info!("Debug program\n");
        log::info!("============================================================\n");

        // load libraries from files
        let libraries = Libraries::new(&self.library_paths)?;

        // load program from file and compile
        let program =
            ProgramFile::read(&self.assembly_file)?.compile(&Debug::On, libraries.libraries)?;

        let program_hash: [u8; 32] = program.hash().into();
        log::info!("Debugging program with hash {}... \n", hex::encode(program_hash));

        // load input data from file
        let input_data = InputFile::read(&self.input_file, &self.assembly_file)?;

        // fetch the stack and program inputs from the arguments
        let stack_inputs = input_data.parse_stack_inputs()?;
        let advice_provider = input_data.parse_advice_provider()?;

        // Instantiate DebugExecutor
        let mut debug_executor = DebugExecutor::new(program, stack_inputs, advice_provider)?;

        // build readline config
        let mut rl_config = Config::builder().auto_add_history(true);
        if self.vim_edit_mode.is_some() {
            rl_config = rl_config.edit_mode(EditMode::Vi);
        }
        let rl_config = rl_config.build();

        // initialize readline
        let mut rl =
            DefaultEditor::with_config(rl_config).expect("Readline couldn't be initialized");

        log::info!("Welcome! Enter `h` for help.\n");

        loop {
            match rl.readline(">> ") {
                Ok(command) => match DebugCommand::parse(&command) {
                    Ok(Some(command)) => {
                        if !debug_executor.execute(command) {
                            log::info!("Debugging complete\n");
                            break;
                        }
                    }
                    Ok(None) => (),
                    Err(err) => eprintln!("{err}"),
                },
                Err(ReadlineError::Interrupted) => {
                    // ctrl+c is a transparent interruption and should provide not feedback or
                    // action.
                }
                Err(ReadlineError::Eof) => {
                    eprintln!("CTRL-D");
                    break;
                }
                Err(err) => eprintln!("malformed command - failed to read user input: {}", err),
            }
        }

        Ok(())
    }
}
