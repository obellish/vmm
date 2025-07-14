mod opt;
mod parse;

use std::{fs, path::PathBuf};

use anyhow::Result;
use clap::Parser;
use koopa::{
	back::KoopaGenerator,
	opt::{Pass, PassManager},
};

use self::{opt::ConstantFolding, parse::parse_program};

fn main() -> Result<()> {
	_ = fs::remove_dir_all("../../out");

	fs::create_dir_all("../../out")?;

	let args = match Args::try_parse() {
		Ok(x) => x,
		Err(e) => {
			eprintln!("{e}");
			return Ok(());
		}
	};

	let input_data = fs::read(&args.file_path)?;

	let mut program = parse_program(input_data)?;

	{
		let out_file = fs::OpenOptions::new()
			.create(true)
			.truncate(true)
			.write(true)
			.open("../../out/unoptimized.ir")?;

		KoopaGenerator::new(out_file).generate_on(&program)?;
	}

	let mut pass_manager = PassManager::new();
	pass_manager.register(Pass::Function(Box::new(ConstantFolding::new())));
	pass_manager.run_passes(&mut program);

	{
		let out_file = fs::OpenOptions::new()
			.create(true)
			.truncate(true)
			.write(true)
			.open("../../out/optimized.ir")?;

		KoopaGenerator::new(out_file).generate_on(&program)?;
	}

	Ok(())
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Args {
	pub file_path: PathBuf,
}
