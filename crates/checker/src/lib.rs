#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]
#![allow(internal_features)]
#![feature(rustc_private, array_chunks, box_patterns, core_intrinsics, f16)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_attr_parsing;
extern crate rustc_data_structures;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_index;
extern crate rustc_interface;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_target;
extern crate rustc_trait_selection;

pub mod call_graph;
