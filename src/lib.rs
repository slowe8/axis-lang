pub mod backend;
mod backend_contract;
#[cfg(feature = "llvm-native")]
mod backend_native;
pub mod borrow;
pub mod diagnostics;
pub mod frontend;
pub mod hir;
pub mod mir;
pub mod passes;
pub mod resolution;
pub mod runtime;
pub mod type_checker;
pub mod types;
