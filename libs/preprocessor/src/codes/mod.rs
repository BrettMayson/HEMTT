//! Preprocessor error codes.

#![allow(missing_docs)]

pub mod pe10_function_as_value;
pub mod pe11_expected_function_or_value;
pub mod pe12_include_not_found;
pub mod pe13_include_not_encased;
pub mod pe14_include_unexpected_suffix;
pub mod pe15_if_invalid_operator;
pub mod pe16_if_incompatible_types;
pub mod pe17_double_else;
pub mod pe18_eoi_ifstate;
pub mod pe19_pragma_unknown;
pub mod pe1_unexpected_token;
pub mod pe20_pragma_invalid_scope;
pub mod pe21_pragma_invalid_suppress;
pub mod pe22_pragma_invalid_flag;
pub mod pe23_if_has_include;
pub mod pe24_parsing_failed;
pub mod pe2_unexpected_eof;
pub mod pe3_expected_ident;
pub mod pe4_unknown_directive;
pub mod pe5_define_multitoken_argument;
pub mod pe6_change_builtin;
pub mod pe7_if_unit_or_function;
pub mod pe8_if_undefined;
pub mod pe9_function_call_argument_count;

pub mod pw1_redefine;
pub mod pw2_invalid_config_case;
pub mod pw3_padded_arg;
