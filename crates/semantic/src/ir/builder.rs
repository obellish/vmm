use super::*;

#[repr(transparent)]
pub struct LocalBuilder<'a> {
	pub(crate) dfg: &'a mut DataFlowGraph,
}

impl BasicBlockBuilder for LocalBuilder<'_> {
	fn insert_basic_block(&mut self, data: BasicBlockData) -> BasicBlock {
		todo!()
	}
}

impl DataFlowGraphInfoQuerier for LocalBuilder<'_> {
	fn data_flow_graph(&self) -> &DataFlowGraph {
		self.dfg
	}
}

impl LocalInstBuilder for LocalBuilder<'_> {}

impl ValueInserter for LocalBuilder<'_> {
	fn insert_value(&mut self, data: ValueData) -> Value {
		self.dfg.insert_value_data(data)
	}
}

impl ValueBuilder for LocalBuilder<'_> {}

pub trait EntityInfoQuerier {
	fn value_type(&self, value: Value) -> Type;

	fn is_const(&self, value: Value) -> bool;

	fn basic_block_params(&self, basic_block: BasicBlock) -> &[Value];

	fn func_type(&self, func: Function) -> Type;
}

pub trait ValueInserter {
	fn insert_value(&mut self, data: ValueData) -> Value;
}

pub trait ValueBuilder: EntityInfoQuerier + ValueInserter + Sized {
	fn raw(&mut self, data: ValueData) -> Value {
		self.insert_value(data)
	}

	fn integer(&mut self, value: i32) -> Value {
		self.insert_value(Integer::into_value_data(Type::i32(), value))
	}

	fn zero_init(&mut self, ty: Type) -> Value {
		self.insert_value(ZeroInit::into_value_data(ty))
	}

	fn undef(&mut self, ty: Type) -> Value {
		self.insert_value(Undef::into_value_data(ty))
	}

	fn aggregate(&mut self, elems: impl IntoIterator<Item = Value>) -> Value {
		let elems = elems.into_iter().collect::<Vec<_>>();
		assert!(!elems.is_empty());
		assert!(elems.iter().all(|e| self.is_const(*e)));

		assert!(
			elems
				.windows(2)
				.all(|e| self.value_type(e[0]) == self.value_type(e[1]))
		);

		let base = self.value_type(elems[0]);
		assert!(!base.is_unit());

		self.insert_value(Aggregate::into_value_data(
			base.into_array(elems.len()),
			elems,
		))
	}
}

pub trait GlobalInstBuilder: ValueBuilder {
	fn global_alloc(&mut self, init: Value) -> Value {
		let init_ty = self.value_type(init);
		assert!(!init_ty.is_unit());
		let ty = Type::ptr(init_ty);
		self.insert_value(GlobalAlloc::into_value_data(ty, init))
	}
}

pub trait LocalInstBuilder: ValueBuilder {
	fn alloc(&mut self, ty: Type) -> Value {
		assert!(!ty.is_unit());
		self.insert_value(Alloc::into_value_data(ty.into_ptr()))
	}

	fn load(&mut self, src: Value) -> Value {
		let src_ty = self.value_type(src);
		let ty = match &*src_ty {
			TypeKind::Pointer(ty) => ty.clone(),
			_ => panic!("expected a ptr type"),
		};

		self.insert_value(Load::into_value_data(ty, src))
	}

	fn store(&mut self, value: Value, dest: Value) -> Value {
		assert_eq!(self.value_type(value).into_ptr(), self.value_type(dest));
		self.insert_value(Store::into_value_data(Type::unit(), (value, dest)))
	}

	fn get_ptr(&mut self, src: Value, index: Value) -> Value {
		let src_ty = self.value_type(src);
		assert!(matches!(&*src_ty, TypeKind::Pointer(..)));
		assert!(self.value_type(index).is_i32());

		self.insert_value(GetPtr::into_value_data(src_ty, (src, index)))
	}

	fn get_element_ptr(&mut self, src: Value, index: Value) -> Value {
		assert!(self.value_type(index).is_i32());

		let ty = match &*self.value_type(src) {
			TypeKind::Pointer(ty) => match &**ty {
				TypeKind::Array(base, ..) => base.clone().into_ptr(),
				_ => panic!("expected a pointer to an array"),
			},
			_ => panic!("expected a pointer to an array"),
		};

		self.insert_value(GetElementPtr::into_value_data(ty, (src, index)))
	}

	fn binary(&mut self, lhs: Value, op: BinaryOp, rhs: Value) -> Value {
		let lhs_ty = self.value_type(lhs);
		let rhs_ty = self.value_type(rhs);

		assert!(lhs_ty.is_i32() && rhs_ty.is_i32());

		self.insert_value(Binary::into_value_data(Type::i32(), (lhs, op, rhs)))
	}

	fn branch(
		&mut self,
		cond: Value,
		true_basic_block: BasicBlock,
		false_basic_block: BasicBlock,
	) -> Value {
		assert!(self.value_type(cond).is_i32());
		assert!(self.basic_block_params(true_basic_block).is_empty());
		assert!(self.basic_block_params(false_basic_block).is_empty());

		self.insert_value(Branch::into_value_data(
			Type::i32(),
			(cond, true_basic_block, false_basic_block),
		))
	}

	fn branch_with_args(
		&mut self,
		cond: Value,
		true_basic_block: BasicBlock,
		false_basic_block: BasicBlock,
		true_args: impl IntoIterator<Item = Value>,
		false_args: impl IntoIterator<Item = Value>,
	) -> Value {
		assert!(self.value_type(cond).is_i32());
		let true_args = true_args.into_iter().collect::<Vec<Value>>();
		let false_args = false_args.into_iter().collect::<Vec<Value>>();

		assert!(
			true_basic_block != false_basic_block
				|| (true_args.is_empty() && false_args.is_empty())
		);

		check_basic_block_arg_types(self, self.basic_block_params(true_basic_block), &true_args);
		check_basic_block_arg_types(
			self,
			self.basic_block_params(false_basic_block),
			&false_args,
		);

		self.insert_value(Branch::into_value_data(
			Type::i32(),
			(
				cond,
				true_basic_block,
				false_basic_block,
				true_args,
				false_args,
			),
		))
	}

	fn jump(&mut self, target: BasicBlock) -> Value {
		assert!(self.basic_block_params(target).is_empty());
		self.insert_value(Jump::into_value_data(Type::unit(), target))
	}

	fn jump_with_args(
		&mut self,
		target: BasicBlock,
		args: impl IntoIterator<Item = Value>,
	) -> Value {
		let args = args.into_iter().collect::<Vec<Value>>();
		check_basic_block_arg_types(self, self.basic_block_params(target), &args);
		self.insert_value(Jump::into_value_data(Type::unit(), (target, args)))
	}

	fn call(&mut self, callee: Function, args: impl IntoIterator<Item = Value>) -> Value {
		let args = args.into_iter().collect::<Vec<Value>>();

		let ty = match &*self.func_type(callee) {
			TypeKind::Function(params, ret) => {
				assert!(
					params
						.iter()
						.zip(args.iter())
						.all(|(ty, a)| ty == &self.value_type(*a))
				);
				ret.clone()
			}
			_ => panic!("expected a function type"),
		};

		self.insert_value(Call::into_value_data(ty, (callee, args)))
	}

	fn ret(&mut self, value: Option<Value>) -> Value {
		assert!(value.is_none_or(|v| !self.value_type(v).is_unit()));
		self.insert_value(Return::into_value_data(Type::unit(), value))
	}
}

pub trait BasicBlockBuilder: ValueInserter + Sized {
	fn insert_basic_block(&mut self, data: BasicBlockData) -> BasicBlock;

	fn basic_block(&mut self, name: Option<impl Into<String>>) -> BasicBlock {
		let name = name.map(Into::into);
		check_basic_block_name(name.as_ref());
		self.insert_basic_block(BasicBlockData::new(name))
	}

	fn basic_block_with_params(
		&mut self,
		name: Option<impl Into<String>>,
		params_ty: impl IntoIterator<Item = Type>,
	) -> BasicBlock {
		let name = name.map(Into::into);
		check_basic_block_name(name.as_ref());
		let params_ty = params_ty.into_iter().collect::<Vec<_>>();
		assert!(params_ty.iter().all(|p| !p.is_unit()));
		let params = params_ty
			.iter()
			.enumerate()
			.map(|(i, ty)| self.insert_value(BlockArgRef::into_value_data(ty.clone(), i)))
			.collect::<Vec<_>>();

		self.insert_basic_block(BasicBlockData::with_params(name, params))
	}

	fn basic_block_with_param_names(
		&mut self,
		name: Option<impl Into<String>>,
		params: impl IntoIterator<Item = (Option<String>, Type)>,
	) -> BasicBlock {
		let name = name.map(Into::into);
		check_basic_block_name(name.as_ref());
		let params = params.into_iter().collect::<Vec<_>>();
		assert!(params.iter().all(|(_, p)| !p.is_unit()));
		let params = params
			.into_iter()
			.enumerate()
			.map(|(i, (n, ty))| {
				let mut arg = BlockArgRef::into_value_data(ty, i);
				arg.set_name(n);
				self.insert_value(arg)
			})
			.collect::<Vec<_>>();

		self.insert_basic_block(BasicBlockData::with_params(name, params))
	}
}

pub trait DataFlowGraphInfoQuerier {
	fn data_flow_graph(&self) -> &DataFlowGraph;
}

impl<T: DataFlowGraphInfoQuerier> EntityInfoQuerier for T {
	fn value_type(&self, value: Value) -> Type {
		self.data_flow_graph()
			.globals
			.upgrade()
			.unwrap()
			.borrow()
			.get(&value)
			.or_else(|| todo!())
			.expect("value does not exist")
			.ty()
			.clone()
	}

	fn is_const(&self, value: Value) -> bool {
		todo!()
	}

	fn basic_block_params(&self, basic_block: BasicBlock) -> &[Value] {
		todo!()
	}

	fn func_type(&self, func: Function) -> Type {
		todo!()
	}
}

fn check_basic_block_arg_types<E: EntityInfoQuerier>(
	querier: &E,
	params: &[Value],
	args: &[Value],
) {
	assert_eq!(params.len(), args.len());
	assert!(
		params
			.iter()
			.zip(args.iter())
			.all(|(p, a)| querier.value_type(*p) == querier.value_type(*a))
	);
}

fn check_basic_block_name(name: Option<&String>) {
	assert!(name.is_none_or(|n| n.len() > 1 && n.starts_with(['%', '@'])));
}
