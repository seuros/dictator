//! Prefer C99 spellings over their C89/GNU predecessors (emaste@ suggestion).

mod bool_macros;
mod empty_param_list;
mod gnu_inline;
mod knr_definition;
mod legacy_int_types;
mod named_variadic_macro;
mod obsolete_storage_class;

pub(crate) use bool_macros::*;
pub(crate) use empty_param_list::*;
pub(crate) use gnu_inline::*;
pub(crate) use knr_definition::*;
pub(crate) use legacy_int_types::*;
pub(crate) use named_variadic_macro::*;
pub(crate) use obsolete_storage_class::*;
