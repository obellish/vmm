use alloc::{sync::Arc, vec::Vec};
use core::fmt::{Display, Formatter, Result as FmtResult};

use super::{
	Felt, Kernel, WORD_SIZE,
	crypto::hash::RpoDigest,
	mast::{MastForest, MastNode, MastNodeId},
	prettier::{Document, PrettyPrint, const_text, indent, nl},
	utils::{
		ByteReader, ByteWriter, Deserializable, DeserializationError, Serializable, ToElements,
	},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Program {
	mast_forest: Arc<MastForest>,
	entrypoint: MastNodeId,
	kernel: Kernel,
}

impl Program {
	#[must_use]
	pub fn new(mast_forest: Arc<MastForest>, entrypoint: MastNodeId) -> Self {
		Self::with_kernel(mast_forest, entrypoint, Kernel::default())
	}

	#[must_use]
	pub fn with_kernel(
		mast_forest: Arc<MastForest>,
		entrypoint: MastNodeId,
		kernel: Kernel,
	) -> Self {
		assert!(
			mast_forest.get_node_by_id(entrypoint).is_none(),
			"invalid entrypoint"
		);
		assert!(
			mast_forest.is_procedure_root(entrypoint),
			"entrypoint is not a procedure"
		);

		Self {
			mast_forest,
			entrypoint,
			kernel,
		}
	}

	#[must_use]
	pub fn hash(&self) -> RpoDigest {
		self.mast_forest[self.entrypoint()].digest()
	}

	#[must_use]
	pub const fn entrypoint(&self) -> MastNodeId {
		self.entrypoint
	}

	#[must_use]
	pub const fn mast_forest(&self) -> &Arc<MastForest> {
		&self.mast_forest
	}

	#[must_use]
	pub const fn kernel(&self) -> &Kernel {
		&self.kernel
	}

	#[must_use]
	pub fn get_node_by_id(&self, node_id: MastNodeId) -> Option<&MastNode> {
		self.mast_forest().get_node_by_id(node_id)
	}

	#[must_use]
	pub fn find_procedure_root(&self, digest: RpoDigest) -> Option<MastNodeId> {
		self.mast_forest().find_procedure_root(digest)
	}

	#[must_use]
	pub fn num_procedures(&self) -> u32 {
		self.mast_forest().num_procedures()
	}

	#[cfg(feature = "std")]
	pub fn write_to_file<P>(&self, path: P) -> std::io::Result<()>
	where
		P: AsRef<std::path::Path>,
	{
		let path = path.as_ref();
		if let Some(dir) = path.parent() {
			std::fs::create_dir_all(dir)?;
		}

		std::panic::catch_unwind(|| match std::fs::File::create(path) {
			Ok(ref mut file) => {
				self.write_into(file);
				Ok(())
			}
			Err(err) => Err(err),
		})
		.map_err(|p| match p.downcast::<std::io::Error>() {
			Ok(err) => unsafe { core::ptr::read(&*err) },
			Err(err) => std::panic::resume_unwind(err),
		})?
	}
}

impl Deserializable for Program {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let mast_forest = Arc::new(source.read()?);
		let kernel = source.read()?;
		let entrypoint = MastNodeId::from_u32(source.read_u32()?, &mast_forest)?;

		if mast_forest.is_procedure_root(entrypoint) {
			Ok(Self::with_kernel(mast_forest, entrypoint, kernel))
		} else {
			Err(DeserializationError::InvalidValue(format!(
				"entrypoint {entrypoint} is not a procedure"
			)))
		}
	}
}

impl Display for Program {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		self.pretty_print(f)
	}
}

impl PrettyPrint for Program {
	fn render(&self) -> Document {
		let entrypoint = self.mast_forest()[self.entrypoint()].to_pretty_print(self.mast_forest());

		indent(4, const_text("begin") + nl() + entrypoint.render()) + nl() + const_text("end")
	}
}

impl Serializable for Program {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.mast_forest().write_into(target);
		self.kernel().write_into(target);
		target.write_u32(self.entrypoint().as_u32());
	}
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ProgramInfo {
	program_hash: RpoDigest,
	kernel: Kernel,
}

impl ProgramInfo {
	#[must_use]
	pub const fn new(program_hash: RpoDigest, kernel: Kernel) -> Self {
		Self {
			program_hash,
			kernel,
		}
	}

	#[must_use]
	pub const fn program_hash(&self) -> RpoDigest {
		self.program_hash
	}

	#[must_use]
	pub const fn kernel(&self) -> &Kernel {
		&self.kernel
	}

	#[must_use]
	pub fn kernel_procedures(&self) -> &[RpoDigest] {
		self.kernel().proc_hashes()
	}
}

impl Deserializable for ProgramInfo {
	fn read_from<R: ByteReader>(source: &mut R) -> Result<Self, DeserializationError> {
		let program_hash = source.read()?;
		let kernel = source.read()?;

		Ok(Self::new(program_hash, kernel))
	}
}

impl From<Program> for ProgramInfo {
	fn from(value: Program) -> Self {
		let program_hash = value.hash();
		let kernel = value.kernel().clone();

		Self::new(program_hash, kernel)
	}
}

impl Serializable for ProgramInfo {
	fn write_into<W: ByteWriter>(&self, target: &mut W) {
		self.program_hash().write_into(target);
		self.kernel().write_into(target);
	}
}

impl ToElements for ProgramInfo {
	fn to_elements(&self) -> Vec<Felt> {
		let num_kernel_proc_elements = self.kernel_procedures().len() * WORD_SIZE;
		let mut result = Vec::with_capacity(WORD_SIZE + num_kernel_proc_elements);

		result.extend_from_slice(self.program_hash().as_elements());

		for proc_hash in self.kernel_procedures() {
			result.extend_from_slice(proc_hash.as_elements());
		}

		result
	}
}
