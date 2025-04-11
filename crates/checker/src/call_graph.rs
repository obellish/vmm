use std::{
	collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
	fmt::{Display, Formatter, Result as FmtResult},
	fs,
	path::Path,
};

use petgraph::{
	Direction, Graph,
	dot::{Config, Dot},
	graph::{DefaultIx, NodeIndex},
	visit::Bfs,
};
use rustc_hir::def_id::DefId;
use rustc_middle::ty::TyCtxt;
use rustc_span::Span;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};

type TypeId = u32;

type NodeId = NodeIndex<DefaultIx>;

type HalfRawEdge = (NodeId, TypeId);
type RawEdge = (NodeId, NodeId, TypeId);
type MidpointExcludedMap = HashMap<NodeId, (HashSet<HalfRawEdge>, HashSet<HalfRawEdge>)>;

const COLLECTION_TYPES: &[&str] = &[
	"std::slice::Iter",
	"std::iter::Enumerate",
	"std::iter::Map",
	"std::collections::HashSet",
	"std::collections::HashMap",
	"std::vec::Vec",
];

pub struct CallGraph<'tcx> {
	pub config: CallGraphConfig,
	tcx: TyCtxt<'tcx>,
	non_local_defs: HashSet<DefId>,
	call_sites: HashMap<rustc_span::Span, (DefId, DefId)>,
	graph: Graph<CallGraphNode, CallGraphEdge>,
	nodes: HashMap<DefId, NodeId>,
	edge_types: HashMap<Box<str>, EdgeType>,
	dominance: HashMap<DefId, HashSet<DefId>>,
}

impl<'tcx> CallGraph<'tcx> {
	#[must_use]
	pub fn new(path_to_config: Option<String>, tcx: TyCtxt<'tcx>) -> Self {
		let config = match path_to_config {
			Some(path) => Self::parse_config(path),
			None => CallGraphConfig::default(),
		};

		Self {
			config,
			tcx,
			non_local_defs: HashSet::new(),
			call_sites: HashMap::new(),
			graph: Graph::new(),
			nodes: HashMap::new(),
			edge_types: HashMap::new(),
			dominance: HashMap::new(),
		}
	}

	fn parse_config(path_to_config: impl AsRef<Path>) -> CallGraphConfig {
		let config_result = fs::read_to_string(path_to_config)
			.map_err(|e| e.to_string())
			.and_then(|config_str| {
				serde_json::from_str::<CallGraphConfig>(&config_str).map_err(|e| e.to_string())
			});

		match config_result {
			Ok(config) => config,
			Err(e) => panic!("failed to read call graph config: {e:?}"),
		}
	}

    #[must_use]
    pub const fn needs_edges(&self) -> bool {
        self.config.dot_output_path.is_some() || self.config.datalog_config.is_some()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CallGraphConfig {
	call_sites_output_path: Option<Box<str>>,
	dot_output_path: Option<Box<str>>,
	reductions: Vec<CallGraphReduction>,
	included_crates: Vec<Box<str>>,
	datalog_config: Option<DatalogConfig>,
	pub include_calls_in_summaries: bool,
}

impl CallGraphConfig {
	#[must_use]
	pub const fn new(
		call_sites_output_path: Option<Box<str>>,
		dot_output_path: Option<Box<str>>,
		reductions: Vec<CallGraphReduction>,
		included_crates: Vec<Box<str>>,
		datalog_config: Option<DatalogConfig>,
	) -> Self {
		Self {
			call_sites_output_path,
			dot_output_path,
			reductions,
			included_crates,
			datalog_config,
			include_calls_in_summaries: false,
		}
	}

	#[must_use]
	pub fn call_sites_path(&self) -> Option<&str> {
		self.call_sites_output_path.as_deref()
	}

	#[must_use]
	pub fn dot_path(&self) -> Option<&str> {
		self.dot_output_path.as_deref()
	}

	#[must_use]
	pub fn ddlog_path(&self) -> Option<&str> {
		self.datalog_config.as_ref().map(DatalogConfig::ddlog_path)
	}

	#[must_use]
	pub fn type_map_path(&self) -> Option<&str> {
		self.datalog_config
			.as_ref()
			.map(DatalogConfig::type_map_path)
	}

	#[must_use]
	pub fn datalog_backend(&self) -> Option<DatalogBackend> {
		self.datalog_config
			.as_ref()
			.map(DatalogConfig::datalog_backend)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatalogConfig {
	ddlog_output_path: Box<str>,
	type_map_output_path: Box<str>,
	type_relations_path: Option<Box<str>>,
	datalog_backend: DatalogBackend,
}

impl DatalogConfig {
	#[must_use]
	pub const fn new(
		ddlog_output_path: Box<str>,
		type_map_output_path: Box<str>,
		type_relations_path: Option<Box<str>>,
		datalog_backend: DatalogBackend,
	) -> Self {
		Self {
			ddlog_output_path,
			type_map_output_path,
			type_relations_path,
			datalog_backend,
		}
	}

	#[must_use]
	pub fn ddlog_path(&self) -> &str {
		&self.ddlog_output_path
	}

	#[must_use]
	pub fn type_map_path(&self) -> &str {
		&self.type_map_output_path
	}

	#[must_use]
	pub fn type_relations_path(&self) -> Option<&str> {
		self.type_relations_path.as_deref()
	}

	#[must_use]
	pub const fn datalog_backend(&self) -> DatalogBackend {
		self.datalog_backend
	}
}

#[derive(Serialize, Deserialize)]
struct Callable {
	name: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	file_index: Option<usize>,
	#[serde(skip_serializing_if = "Option::is_none")]
	first_line: Option<usize>,
	local: bool,
}

#[derive(Serialize, Deserialize)]
struct CallSiteOutput {
	#[serde(skip_serializing_if = "Vec::is_empty")]
	files: Vec<String>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	callables: Vec<Callable>,
	#[serde(skip_serializing_if = "Vec::is_empty")]
	calls: Vec<(usize, usize, usize, usize, usize)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
struct TypeRelation {
	kind: TypeRelationKind,
	type1: Box<str>,
	type2: Box<str>,
}

impl TypeRelation {
	const fn eq(t1: Box<str>, t2: Box<str>) -> Self {
		Self {
			kind: TypeRelationKind::Eq,
			type1: t1,
			type2: t2,
		}
	}

	const fn member(t1: Box<str>, t2: Box<str>) -> Self {
		Self {
			kind: TypeRelationKind::Member,
			type1: t1,
			type2: t2,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DatalogRelation {
	name: RelationType,
	operands: Vec<u32>,
}

#[repr(transparent)]
struct DatalogOutput {
	relations: HashSet<DatalogRelation>,
}

#[derive(Serialize, Deserialize)]
#[repr(transparent)]
struct TypeRelationsRaw {
	relations: Vec<TypeRelation>,
}

#[repr(transparent)]
struct TypeMapOutput {
	map: HashMap<TypeId, Box<str>>,
}

#[derive(Debug, Clone)]
struct EdgeType {
	id: TypeId,
	name: Box<str>,
}

impl EdgeType {
	const fn new(id: TypeId, name: Box<str>) -> Self {
		Self { id, name }
	}
}

#[derive(Debug, Clone)]
struct CallGraphNode {
	defid: DefId,
	name: Box<str>,
	node_type: NodeType,
}

impl CallGraphNode {
	fn croot(defid: DefId) -> Self {
		Self {
			defid,
			name: Self::format_name(defid),
			node_type: NodeType::CRoot,
		}
	}

	fn root(defid: DefId) -> Self {
		Self {
			defid,
			name: Self::format_name(defid),
			node_type: NodeType::Root,
		}
	}

	fn format_name(defid: DefId) -> Box<str> {
		let tmp1 = format!("{defid:?}");
		let tmp2 = tmp1.split("~ ").collect::<Vec<&str>>()[1];
		let tmp3 = tmp2.replace(')', "");
		let lhs = tmp3.split('[').collect::<Vec<&str>>()[0];
		let rhs = tmp3.split(']').collect::<Vec<&str>>()[1];
		format!("{lhs}{rhs}").into_boxed_str()
	}

	fn is_excluded(&self, included_crates: &[&str]) -> bool {
		let mut excluded = true;
		for crate_name in included_crates {
			if self.name.contains(crate_name) {
				excluded = false;
				break;
			}
		}

		excluded
	}

	const fn is_croot(&self) -> bool {
		matches!(self.node_type, NodeType::CRoot)
	}
}

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct CallGraphEdge {
	type_id: TypeId,
}

impl CallGraphEdge {
	const fn new(type_id: TypeId) -> Self {
		Self { type_id }
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallGraphReduction {
	Slice(Box<str>),
	Fold,
	Deduplicate,
	Clean,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DatalogBackend {
	DifferentialDatalog,
	Souffle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RelationType {
	Dom,
	Edge,
	EdgeType,
	EqType,
	Member,
}

impl Display for RelationType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Dom => "Dom",
			Self::Edge => "Edge",
			Self::EdgeType => "EdgeType",
			Self::EqType => "EqType",
			Self::Member => "Member",
		})
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum TypeRelationKind {
	Eq,
	Member,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum SimpleType {
	Base(Box<str>),
	Collection(Box<str>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeType {
	Root,
	CRoot,
}
