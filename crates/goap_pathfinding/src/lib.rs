#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

pub mod directed;
pub mod grid;
pub mod kuhn_munkres;
pub mod matrix;
mod noderefs;
pub mod prelude;
pub mod undirected;
pub mod utils;

use std::hash::BuildHasherDefault;

use indexmap::{IndexMap, IndexSet};
pub use num_traits;
use rustc_hash::FxHasher;

pub use self::noderefs::NodeRefs;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type FxIndexSet<V> = IndexSet<V, BuildHasherDefault<FxHasher>>;
