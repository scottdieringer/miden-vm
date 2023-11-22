use winter_utils::{
    collections::{vec, Vec},
    ByteWriter, Serializable,
};

use super::{
    errors::{InputError, OutputError},
    Felt, StackTopState, StarkField, ToElements,
};

mod inputs;
pub use inputs::StackInputs;

mod outputs;
pub use outputs::StackOutputs;

// CONSTANTS
// ================================================================================================

/// The number of stack registers which can be accessed by the VM directly. This is also the
/// minimum stack depth enforced by the VM.
pub const STACK_TOP_SIZE: usize = 16;
