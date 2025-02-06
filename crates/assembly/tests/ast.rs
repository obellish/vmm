use std::sync::Arc;

use vmm_assembly::{
	Span,
	ast::*,
	diagnostics::{Report, reporting::PrintDiagnostic},
};
use vmm_core::Felt;

macro_rules! id {
	($name:ident) => {
		Ident::new(stringify!($name)).unwrap()
	};
}

macro_rules! inst {
	($inst:ident($value:expr)) => {
		Op::Inst(Span::unknown(Instruction::$inst($value)))
	};
	($inst:ident) => {
		Op::Inst(Span::unknown(Instruction::$inst))
	};
}

macro_rules! exec {
	($name:ident) => {
		inst!(Exec(InvocationTarget::ProcedureName(
			stringify!($name).parse().expect("invalid procedure name")
		)))
	};

	($name:path) => {{
		let path = stringify!($name);
		let (module, name) = path.split_once("::").expect("invalid procedure path");
		let name =
			Ident::new_unchecked(Span::unknown(Arc::from(name.to_string().into_boxed_str())));
		let name = ProcedureName::new_unchecked(name);

		inst!(Exec(InvocationTarget::ProcedurePath {
			name,
			module: module.parse().unwrap()
		}))
	}};
}

#[allow(unused_macros)]
macro_rules! call {
	($name:ident) => {
		inst!(Call(InvocationTarget::ProcedureName(
			stringify!($name).parse()
		)))
	};

	($name:path) => {{
		let path = stringify!($name);
		let (module, name) = path.split_once("::").expect("invalid procedure path");
		let name = ProcedureName::new_unchecked(Default::default(), name);

		inst!(Call(InvocationTarget::ProcedurePath { name, module }))
	}};
}

macro_rules! block {
    ($($insts:expr),+) => {
        Block::new(Default::default(), Vec::from([$($insts),*]))
    }
}

macro_rules! moduledoc {
	($doc:literal) => {
		Form::ModuleDoc(Span::unknown($doc.to_string()))
	};

	($doc:ident) => {
		Form::ModuleDoc(Span::unknown($doc.to_string()))
	};
}

macro_rules! doc {
	($doc:literal) => {
		Form::Doc(Span::unknown($doc.to_string()))
	};

	($doc:ident) => {
		Form::Doc(Span::unknown($doc.to_string()))
	};
}

macro_rules! begin {
    ($($insts:expr),+) => {
        Form::Begin(block!($($insts),*))
    }
}

macro_rules! if_true {
	($then_blk:expr) => {
		Op::If {
			span: Default::default(),
			then_blk: $then_blk,
			else_blk: Block::default(),
		}
	};

	($then_blk:expr, $else_blk:expr) => {
		Op::If {
			span: Default::default(),
			then_blk: $then_blk,
			else_blk: $else_blk,
		}
	};
}

macro_rules! while_true {
	($body:expr) => {
		Op::While {
			span: Default::default(),
			body: $body,
		}
	};
}

macro_rules! import {
	($name:literal) => {{
		let path: crate::LibraryPath = $name.parse().expect("invalid import path");
		let name = path.last().parse().unwrap();
		Form::Import(Import {
			span: crate::SourceSpan::default(),
			name,
			path,
			uses: 0,
		})
	}};

	($name:literal -> $alias:literal) => {
		let path: LibraryPath = $name.parse().expect("invalid import path");
		let name = $alias.parse().expect("invalid import alias");
		Form::Import(Import {
			span: SourceSpan::default(),
			name,
			path,
			uses: 0,
		})
	};
}

macro_rules! proc {
    ($name:ident, $num_locals:literal, $body:expr) => {
        Form::Procedure(Export::Procedure(Procedure::new(
            Default::default(),
            Visibility::Private,
            stringify!($name).parse().expect("invalid procedure name"),
            $num_locals,
            $body,
        )))
    };

    ([$($attr:expr),*], $name:ident, $num_locals:literal, $body:expr) => {
        Form::Procedure(Export::Procedure(
            Procedure::new(
                Default::default(),
                Visibility::Private,
                stringify!($name).parse().expect("invalid procedure name"),
                $num_locals,
                $body,
            )
            .with_attributes([$($attr),*]),
        ))
    };

    ($docs:literal, $name:ident, $num_locals:literal, $body:expr) => {
        Form::Procedure(Export::Procedure(
            Procedure::new(
                Default::default(),
                Visibility::Private,
                stringify!($name).parse().expect("invalid procedure name"),
                $num_locals,
                $body,
            )
            .with_docs(Some(Span::unknown($docs.to_string()))),
        ))
    };

    ($docs:literal, [$($attr:expr),*], $name:ident, $num_locals:literal, $body:expr) => {
        Form::Procedure(Export::Procedure(
            Procedure::new(
                Default::default(),
                Visibility::Private,
                stringify!($name).parse().expect("invalid procedure name"),
                $num_locals,
                $body,
            )
            .with_docs($docs)
            .with_attributes([$($attr),*]),
        ))
    };
}

macro_rules! export {
	($name:ident, $num_locals:literal, $body:expr) => {
		Form::Procedure(Export::Procedure(Procedure::new(
			Default::default(),
			Visibility::Public,
			stringify!($name).parse().expect("invalid procedure name"),
			$num_locals,
			$body,
		)))
	};

	($docs:expr, $name:ident, $num_locals:literal, $body:expr) => {
		Form::Procedure(Export::Procedure(
			Procedure::new(
				Default::default(),
				Visibility::Public,
				stringify!($name).parse().expect("invalid procedure name"),
				$num_locals,
				$body,
			)
			.with_docs(Some(Span::unknown($docs.to_string()))),
		))
	};
}

macro_rules! module {
    ($($forms:expr),+) => {
        Vec::<Form>::from([
            $(
                Form::from($forms),
            )*
        ])
    }
}

macro_rules! assert_forms {
	($context:ident, $source:expr, $expected:expr) => {
		match $context.parse_forms($source.clone()) {
			Ok(forms) => assert_eq!(forms, $expected),
			Err(report) => {
				panic!(
					"expected parsing to succeed but failed with error:
{}",
					crate::diagnostics::reporting::PrintDiagnostic::new_without_color(report)
				);
			}
		}
	};
}

macro_rules! assert_parse_diagnostic {
	($source:expr, $expected:literal) => {{
		let source = $source.clone();
		let error = crate::parser::parse_forms(source.clone())
			.map_err(|err| Report::new(err).with_source_code(source))
			.expect_err("expected diagnostic to be raised, but parsing succeeded");
		assert_diagnostic!(error, $expected);
	}};

	($source:expr, $expected:expr) => {{
		let source = $source.clone();
		let error = crate::parser::parse_forms(source.clone())
			.map_err(|err| Report::new(err).with_source_code(source))
			.expect_err("expected diagnostic to be raised, but parsing succeeded");
		assert_diagnostic!(error, $expected);
	}};
}

macro_rules! assert_parse_diagnostic_lines {
    ($source:expr, $($expected:literal),+) => {{
        let error = crate::parser::parse_forms(source.clone())
            .map_err(|err| Report::new(err).with_source_code(source))
            .expect_err("expected diagnostic to be raised, but parsing succeeded");
        assert_diagnostic_lines!(error, $($expected),*);
    }};

    ($source:expr, $($expected:expr),+) => {{
        let source = $source.clone();
        let error = crate::parser::parse_forms(source.clone())
            .map_err(|err| Report::new(err).with_source_code(source))
            .expect_err("expected diagnostic to be raised, but parsing succeeded");
        assert_diagnostic_lines!(error, $($expected),*);
    }};
}

macro_rules! assert_module_diagnostic_lines {
    ($context:ident, $source:expr, $($expected:literal),+) => {{
        let error = $context
            .parse_module($source)
            .expect_err("expected diagnostic to be raised, but parsing succeeded");
        assert_diagnostic_lines!(error, $($expected),*);
    }};

    ($context:ident, $source:expr, $($expected:expr),+) => {{
        let error = $context
            .parse_module($source)
            .expect_err("expected diagnostic to be raised, but parsing succeeded");
        assert_diagnostic_lines!(error, $($expected),*);
    }};
}

macro_rules! assert_program_diagnostic_lines {
    ($context:ident, $source:expr, $($expected:literal),+) => {{
        let error = $context
            .parse_program($source)
            .expect_err("expected diagnostic to be raised, but parsing succeeded");
        assert_diagnostic_lines!(error, $($expected),*);
    }};

    ($context:ident, $source:expr, $($expected:expr),+) => {{
        let error = $context
            .parse_program($source)
            .expect_err("expected diagnostic to be raised, but parsing succeeded");
        assert_diagnostic_lines!(error, $($expected),*);
    }};
}

type Result<T = (), E = Report> = std::result::Result<T, E>;

#[test]
fn ast_parsing_program_simple() -> Result {
	Ok(())
}
