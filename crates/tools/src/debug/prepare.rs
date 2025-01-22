use vmm::{
	board::{Bus, MotherBoard},
	mem::{ContiguousMappingResult, MappingRange},
};

#[must_use]
pub fn prepare_vm(components: Vec<Box<dyn Bus>>) -> MotherBoard {
	let aux_count = components.len();

	let mut motherboard = MotherBoard::from_iter(components);

	motherboard.map(|mem| {
		let ContiguousMappingResult {
			mapping,
			aux_mapping,
		} = mem.map_contiguous(0x0000_0000, (0..aux_count).collect::<Vec<_>>());

		for result in aux_mapping {
			println!(
				"=> component {:04} '{:32}': {} {} (HW ID: 0x{})",
				result.aux_id,
				result.aux_name,
				if result.aux_mapping.is_ok() {
					"✓"
				} else {
					"✗"
				},
				match result.aux_mapping {
					Ok(MappingRange {
						start_addr,
						end_addr,
					}) => format!("{start_addr:#010X} -> {end_addr:#010X}"),
					Err(err) => format!("{err:?}"),
				},
				result
					.aux_hw_id
					.to_be_bytes()
					.iter()
					.map(|byte| format!("{byte:002X}"))
					.collect::<Vec<String>>()
					.join(" ")
			);
		}

		if let Err(failed) = mapping {
			panic!(
				"failed to map {} component{}",
				failed.len(),
				if matches!(failed.len(), 1) { "" } else { "s" }
			)
		}
	});

	motherboard.reset();
	motherboard
}
