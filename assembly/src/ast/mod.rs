//! Abstract syntax tree (AST) components of Miden programs, modules, and procedures.
//!
//! Structs in this module (specifically [ProgramAst] and [ModuleAst]) can be used to parse source
//! code into relevant ASTs. This can be done via their `parse()` methods.
pub use tracing::{event, info_span, instrument, Level};

use super::{
    crypto::hash::RpoDigest, BTreeMap, ByteReader, ByteWriter, Deserializable,
    DeserializationError, Felt, LabelError, LibraryPath, ParsingError, ProcedureId, ProcedureName,
    Serializable, SliceReader, StarkField, String, ToString, Token, TokenStream, Vec,
    MAX_LABEL_LEN,
};
use vm_core::utils::bound_into_included_u64;

pub use super::tokens::SourceLocation;

mod nodes;
use nodes::FormattableNode;
pub use nodes::{AdviceInjectorNode, Instruction, Node};

mod code_body;
pub use code_body::CodeBody;

mod format;
use format::*;

mod imports;
pub use imports::ModuleImports;

mod invocation_target;
pub use invocation_target::InvocationTarget;

mod parsers;

mod module;
pub use module::ModuleAst;

mod procedure;
pub use procedure::{ProcReExport, ProcedureAst};

mod program;
pub use program::ProgramAst;

pub(crate) use parsers::{
    parse_param_with_constant_lookup, NAMESPACE_LABEL_PARSER, PROCEDURE_LABEL_PARSER,
};

mod serde;
pub use serde::AstSerdeOptions;

#[cfg(test)]
pub mod tests;

// CONSTANTS
// ================================================================================================

/// Maximum number of procedures in a module.
const MAX_LOCAL_PROCS: usize = u16::MAX as usize;

/// Maximum number of re-exported procedures in a module.
const MAX_REEXPORTED_PROCS: usize = u16::MAX as usize;

/// Maximum number of bytes for a single documentation comment.
const MAX_DOCS_LEN: usize = u16::MAX as usize;

/// Maximum number of nodes in statement body (e.g., procedure body, loop body etc.).
const MAX_BODY_LEN: usize = u16::MAX as usize;

/// Maximum number of imported libraries in a module or a program
const MAX_IMPORTS: usize = u16::MAX as usize;

/// Maximum number of imported procedures used in a module or a program
const MAX_INVOKED_IMPORTED_PROCS: usize = u16::MAX as usize;

/// Maximum stack index at which a full word can start.
const MAX_STACK_WORD_OFFSET: u8 = 12;

// TYPE ALIASES
// ================================================================================================
type LocalProcMap = BTreeMap<ProcedureName, (u16, ProcedureAst)>;
type LocalConstMap = BTreeMap<String, u64>;
type ReExportedProcMap = BTreeMap<ProcedureName, ProcReExport>;
type InvokedProcsMap = BTreeMap<ProcedureId, (ProcedureName, LibraryPath)>;

// HELPER FUNCTIONS
// ================================================================================================

/// Sort a map of procedures into a vec, respecting the order set in the map
fn sort_procs_into_vec(proc_map: LocalProcMap) -> Vec<ProcedureAst> {
    let mut procedures: Vec<_> = proc_map.into_values().collect();
    procedures.sort_by_key(|(idx, _proc)| *idx);

    procedures.into_iter().map(|(_idx, proc)| proc).collect()
}

/// Logging a warning message for every imported but unused module.
#[cfg(feature = "std")]
fn check_unused_imports(import_info: &ModuleImports) {
    let import_lib_paths = import_info.import_paths();
    let invoked_procs_paths: Vec<&LibraryPath> =
        import_info.invoked_procs().iter().map(|(_id, (_name, path))| path).collect();

    for lib in import_lib_paths {
        if !invoked_procs_paths.contains(&lib) {
            event!(Level::WARN, "warning: unused import: \"{}\"", lib);
        }
    }
}
