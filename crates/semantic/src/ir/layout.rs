use std::{
	borrow::Borrow,
	cell::RefCell,
	collections::{HashMap, hash_map::Entry},
	hash::Hash,
	rc::{Rc, Weak},
};

use key_node_list::{KeyNodeList, Map, impl_node};

use super::{BasicBlock, Value};

pub struct Layout {
	inst_basic_block: Rc<RefCell<HashMap<Value, BasicBlock>>>,
	basic_blocks: BasicBlockList,
}

impl Layout {
	#[must_use]
	pub fn new() -> Self {
		let inst_basic_block = Rc::new(RefCell::new(HashMap::new()));
		Self {
			basic_blocks: BasicBlockList::with_map(BasicBlockMap::new(Rc::downgrade(
				&inst_basic_block,
			))),
			inst_basic_block,
		}
	}

	#[must_use]
	pub const fn basic_blocks(&self) -> &BasicBlockList {
		&self.basic_blocks
	}

	pub const fn basic_blocks_mut(&mut self) -> &mut BasicBlockList {
		&mut self.basic_blocks
	}

	#[must_use]
	pub fn entry_basic_block(&self) -> Option<BasicBlock> {
		self.basic_blocks.front_key().copied()
	}

	#[must_use]
	pub fn parent_basic_block(&self, inst: Value) -> Option<BasicBlock> {
		self.inst_basic_block.as_ref().borrow().get(&inst).copied()
	}
}

impl Default for Layout {
	fn default() -> Self {
		Self::new()
	}
}

pub struct BasicBlockMap {
	inst_basic_block: InstBasicBlockCell,
	map: HashMap<BasicBlock, BasicBlockNode>,
}

impl BasicBlockMap {
	fn new(inst_basic_block: InstBasicBlockCell) -> Self {
		Self {
			inst_basic_block,
			map: HashMap::new(),
		}
	}
}

impl Map<BasicBlock, BasicBlockNode> for BasicBlockMap {
	fn len(&self) -> usize {
		self.map.len()
	}

	fn clear(&mut self) {
		self.map.clear();
	}

	fn get<Q>(&self, k: &Q) -> Option<&BasicBlockNode>
	where
		BasicBlock: Borrow<Q>,
		Q: ?Sized + Eq + Hash,
	{
		self.map.get(k)
	}

	fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut BasicBlockNode>
	where
		BasicBlock: Borrow<Q>,
		Q: ?Sized + Eq + Hash,
	{
		self.map.get_mut(k)
	}

	fn insert<T: Into<BasicBlockNode>>(
		&mut self,
		k: BasicBlock,
		v: T,
	) -> Result<(), (BasicBlock, T)> {
		if let Entry::Vacant(e) = self.map.entry(k) {
			let node = BasicBlockNode::new(k, self.inst_basic_block.clone());
			e.insert(node);
			Ok(())
		} else {
			Err((k, v))
		}
	}

	fn remove_entry<Q>(&mut self, k: &Q) -> Option<(BasicBlock, BasicBlockNode)>
	where
		BasicBlock: Borrow<Q>,
		Q: ?Sized + Eq + Hash,
	{
		self.map.remove_entry(k)
	}
}

pub struct BasicBlockNode {
	insts: InstList,
	prev: Option<BasicBlock>,
	next: Option<BasicBlock>,
}

impl BasicBlockNode {
	fn new(basic_block: BasicBlock, inst_basic_block: InstBasicBlockCell) -> Self {
		Self {
			insts: InstList::with_map(InstMap::new(basic_block, inst_basic_block)),
			prev: None,
			next: None,
		}
	}
}

#[allow(clippy::fallible_impl_from)]
impl From<()> for BasicBlockNode {
	fn from((): ()) -> Self {
		panic!("should not be called")
	}
}

impl_node!(BasicBlockNode { Key = BasicBlock, prev = prev, next =  next });

pub struct InstMap {
	basic_block: BasicBlock,
	inst_basic_block: InstBasicBlockCell,
	map: HashMap<Value, InstNode>,
}

impl InstMap {
	fn new(basic_block: BasicBlock, inst_basic_block: InstBasicBlockCell) -> Self {
		Self {
			basic_block,
			inst_basic_block,
			map: HashMap::new(),
		}
	}
}

impl Map<Value, InstNode> for InstMap {
	fn len(&self) -> usize {
		self.map.len()
	}

	fn clear(&mut self) {
		self.map.clear();
	}

	fn get<Q>(&self, k: &Q) -> Option<&InstNode>
	where
		Value: Borrow<Q>,
		Q: ?Sized + Eq + Hash,
	{
		self.map.get(k)
	}

	fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut InstNode>
	where
		Value: Borrow<Q>,
		Q: ?Sized + Eq + Hash,
	{
		self.map.get_mut(k)
	}

	fn insert<T>(&mut self, k: Value, v: T) -> Result<(), (Value, T)>
	where
		T: Into<InstNode>,
	{
		if self.contains_key(&k) {
			Err((k, v))
		} else {
			self.inst_basic_block
				.upgrade()
				.unwrap()
				.as_ref()
				.borrow_mut()
				.insert(k, self.basic_block);
			self.map.insert(k, v.into());
			Ok(())
		}
	}

	fn remove_entry<Q>(&mut self, k: &Q) -> Option<(Value, InstNode)>
	where
		Value: Borrow<Q>,
		Q: ?Sized + Eq + Hash,
	{
		let kv = self.map.remove_entry(k);
		if kv.is_some() {
			self.inst_basic_block
				.upgrade()
				.unwrap()
				.as_ref()
				.borrow_mut()
				.remove(k);
		}

		kv
	}
}

#[derive(Default)]
pub struct InstNode {
	prev: Option<Value>,
	next: Option<Value>,
}

impl From<()> for InstNode {
	fn from((): ()) -> Self {
		Self::default()
	}
}

impl_node!(InstNode { Key = Value, prev = prev, next = next });

pub type BasicBlockList = KeyNodeList<BasicBlock, BasicBlockNode, BasicBlockMap>;

pub type InstList = KeyNodeList<Value, InstNode, InstMap>;

type InstBasicBlockCell = Weak<RefCell<HashMap<Value, BasicBlock>>>;
