use std::{
	fmt::{Display, Formatter, Result as FmtResult},
	path::PathBuf,
};

use clap::{
	Arg, ArgAction, ArgGroup, ArgMatches, Args as ClapArgs, Command, CommandFactory,
	Error as ClapError, FromArgMatches, Parser, ValueEnum,
	builder::{EnumValueParser, PossibleValue, ValueParser},
	error::ErrorKind as ClapErrorKind,
};

#[derive(Debug)]
pub struct Args {
	pub file: PathBuf,
	pub optimize: bool,
	pub tape: TapeType,
}

impl ClapArgs for Args {
	fn augment_args(cmd: Command) -> Command {
		cmd.group(ArgGroup::new("Args").multiple(true).args([
			clap::Id::from("file"),
			clap::Id::from("optimize"),
			clap::Id::from("tape_type"),
		]))
		.arg(
			Arg::new("file")
				.value_name("FILE")
				.value_parser(ValueParser::path_buf())
				.required(ArgAction::Set.takes_values())
				.action(ArgAction::Set),
		)
		.arg(
			Arg::new("optimize")
				.value_name("OPTIMIZE")
				.required(ArgAction::SetTrue.takes_values())
				.value_parser(ValueParser::bool())
				.action(ArgAction::SetTrue)
				.short('o')
				.long("optimize"),
		)
		.arg(
			Arg::new("tape_type")
				.value_name("TAPE_TYPE")
				.required(false)
				.action(ArgAction::Set)
				.value_parser(EnumValueParser::<TapeType>::new())
				.short('t')
				.default_value("ptr")
				.long("tape"),
		)
	}

	fn augment_args_for_update(cmd: Command) -> Command {
		cmd.group(ArgGroup::new("Args").multiple(true).args([
			clap::Id::from("file"),
			clap::Id::from("optimize"),
			clap::Id::from("tape_type"),
		]))
		.arg(
			Arg::new("file")
				.value_name("FILE")
				.value_parser(ValueParser::path_buf())
				.required(false)
				.action(ArgAction::Set),
		)
		.arg(
			Arg::new("optimize")
				.value_name("OPTIMIZE")
				.required(false)
				.value_parser(ValueParser::bool())
				.action(ArgAction::SetTrue)
				.short('o')
				.long("optimize"),
		)
		.arg(
			Arg::new("tape_type")
				.value_name("TAPE_TYPE")
				.required(false)
				.action(ArgAction::Set)
				.value_parser(EnumValueParser::<TapeType>::new())
				.short('t')
				.long("tape"),
		)
	}
}

impl CommandFactory for Args {
	fn command() -> Command {
		Self::augment_args(Command::new("vmm"))
	}

	fn command_for_update() -> Command {
		Self::augment_args_for_update(Command::new("vmm"))
	}
}

impl FromArgMatches for Args {
	fn from_arg_matches(matches: &ArgMatches) -> Result<Self, ClapError> {
		Self::from_arg_matches_mut(&mut matches.clone())
	}

	fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, ClapError> {
		Ok(Self {
			file: matches.remove_one("file").ok_or_else(|| {
				ClapError::raw(
					ClapErrorKind::MissingRequiredArgument,
					"The following required argument was not provided: file",
				)
			})?,
			optimize: matches.remove_one("optimize").ok_or_else(|| {
				ClapError::raw(
					ClapErrorKind::MissingRequiredArgument,
					"The following required argument was not provided: optimize",
				)
			})?,
			tape: matches.remove_one("tape_type").ok_or_else(|| {
				ClapError::raw(
					ClapErrorKind::MissingRequiredArgument,
					"The following required argument was not provided: tape",
				)
			})?,
		})
	}

	fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), ClapError> {
		self.update_from_arg_matches_mut(&mut matches.clone())
	}

	fn update_from_arg_matches_mut(&mut self, matches: &mut ArgMatches) -> Result<(), ClapError> {
		if matches.contains_id("file") {
			let file = &mut self.file;
			*file = matches.remove_one("file").ok_or_else(|| {
				ClapError::raw(
					ClapErrorKind::MissingRequiredArgument,
					"The following required argument was not provided: file",
				)
			})?;
		}

		if matches.contains_id("optimize") {
			let optimize = &mut self.optimize;
			*optimize = matches.remove_one("optimize").ok_or_else(|| {
				ClapError::raw(
					ClapErrorKind::MissingRequiredArgument,
					"The following required argument was not provided: optimize",
				)
			})?;
		}

		if matches.contains_id("tape_type") {
			let tape_type = &mut self.tape;
			*tape_type = matches.remove_one("tape_type").ok_or_else(|| {
				ClapError::raw(
					ClapErrorKind::MissingRequiredArgument,
					"The following required argument was not provided: tape",
				)
			})?;
		}

		Ok(())
	}
}

impl Parser for Args {}

#[derive(Debug, Clone, Copy)]
pub enum TapeType {
	Box,
	Vec,
	Ptr,
	Stack,
}

impl Display for TapeType {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(match self {
			Self::Box => "box",
			Self::Vec => "vec",
			Self::Ptr => "ptr",
			Self::Stack => "stack",
		})
	}
}

impl ValueEnum for TapeType {
	fn value_variants<'a>() -> &'a [Self] {
		&[Self::Box, Self::Vec, Self::Ptr, Self::Stack]
	}

	fn to_possible_value(&self) -> Option<PossibleValue> {
		Some(PossibleValue::new(match self {
			Self::Box => "box",
			Self::Vec => "vec",
			Self::Ptr => "ptr",
			Self::Stack => "stack",
		}))
	}
}
