use crate::Offset;

pub trait IsOffsetable: super::sealed::Sealed {
	fn is_offsetable(&self) -> bool;

	fn offset(&self) -> Option<Offset>;

	fn set_offset(&mut self, offset: Offset);

	fn shift_offset(&mut self, offset: Offset) {
		extern crate std;

		if let Some(self_offset) = self.offset() {
			self.set_offset(self_offset + offset);
		} else {
			self.set_offset(offset);
		}
	}

	unsafe fn offset_unchecked(&self) -> Offset {
		unsafe { self.offset().unwrap_unchecked() }
	}
}

impl<T: IsOffsetable> IsOffsetable for [T] {
	fn is_offsetable(&self) -> bool {
		self.iter().all(IsOffsetable::is_offsetable)
	}

	fn offset(&self) -> Option<Offset> {
		self.iter().try_fold(Offset(0), |a, b| {
			let offset = b.offset()?;

			Some(a + offset)
		})
	}

	fn shift_offset(&mut self, offset: Offset) {
		self.iter_mut().for_each(|v| {
			if let Some(self_offset) = v.offset() {
				v.set_offset(self_offset + offset);
			} else {
				v.set_offset(offset);
			}
		});
	}

	fn set_offset(&mut self, offset: Offset) {
		self.iter_mut().for_each(|v| v.set_offset(offset));
	}
}

#[cfg(test)]
mod tests {
	use crate::{Instruction, IsOffsetable, Offset};

	#[test]
	fn basic() {
		let mut instrs = [Instruction::inc_val(2), Instruction::set_val(4)];

		assert_eq!(instrs.offset(), Some(Offset(0)));

		assert!(instrs.is_offsetable());

		instrs.set_offset(Offset(2));

		assert_eq!(
			instrs,
			[Instruction::inc_val_at(2, 2), Instruction::set_val_at(4, 2)]
		);

		instrs.shift_offset(Offset(-1));

		assert_eq!(
			instrs,
			[Instruction::inc_val_at(2, 1), Instruction::set_val_at(4, 1)]
		);
	}
}
