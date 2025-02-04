use alloc::{borrow::Cow, boxed::Box, sync::Arc, vec::Vec};
use core::{
	error::Error as StdError,
	fmt::{Display, Formatter, Result as FmtResult},
	ops::Range,
};

pub use vmm_core::debuginfo::*;
#[doc(inline)]
pub use {
	miette::{
		self, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Report, Result, Severity,
		SourceCode, WrapErr,
	},
	tracing,
};

pub mod reporting {
	use core::fmt::{Display, Formatter, Result as FmtResult};

	pub use super::miette::{
		DebugReportHandler, JSONReportHandler, NarratableReportHandler, ReportHandler, set_hook,
	};
	#[cfg(feature = "std")]
	pub use super::miette::{GraphicalReportHandler, GraphicalTheme, set_panic_hook};

	pub type ReportHandlerOpts = super::miette::MietteHandlerOpts;

	#[cfg(feature = "std")]
	pub type DefaultReportHandler = GraphicalReportHandler;

	#[cfg(not(feature = "std"))]
	pub type DefaultReportHandler = DebugReportHandler;

	pub struct PrintDiagnostic<D, R = DefaultReportHandler> {
		handler: R,
		diag: D,
	}

	impl<D> PrintDiagnostic<D>
	where
		D: AsRef<dyn super::Diagnostic>,
	{
		pub fn new(diag: D) -> Self {
			Self {
				handler: DefaultReportHandler::default(),
				diag,
			}
		}

		#[cfg(feature = "std")]
		pub fn without_color(diag: D) -> Self {
			Self {
				handler: DefaultReportHandler::new_themed(GraphicalTheme::none()),
				diag,
			}
		}

		#[cfg(not(feature = "std"))]
		pub fn without_color(diag: D) -> Self {
			Self::new(diag)
		}
	}

	impl<D> PrintDiagnostic<D, NarratableReportHandler>
	where
		D: AsRef<dyn super::Diagnostic>,
	{
		pub fn narrated(diag: D) -> Self {
			Self {
				handler: NarratableReportHandler::default(),
				diag,
			}
		}
	}

	impl<D> PrintDiagnostic<D, JSONReportHandler>
	where
		D: AsRef<dyn super::Diagnostic>,
	{
		pub const fn json(diag: D) -> Self {
			Self {
				handler: JSONReportHandler,
				diag,
			}
		}
	}

	impl<D> Display for PrintDiagnostic<D>
	where
		D: AsRef<dyn super::Diagnostic>,
	{
		fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
			self.handler.render_report(f, self.diag.as_ref())
		}
	}

	impl<D> Display for PrintDiagnostic<D, NarratableReportHandler>
	where
		D: AsRef<dyn super::Diagnostic>,
	{
		fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
			self.handler.render_report(f, self.diag.as_ref())
		}
	}

	impl<D> Display for PrintDiagnostic<D, JSONReportHandler>
	where
		D: AsRef<dyn super::Diagnostic>,
	{
		fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
			self.handler.render_report(f, self.diag.as_ref())
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
	span: miette::SourceSpan,
	label: Option<Cow<'static, str>>,
}

impl Label {
	pub fn at(range: impl Into<Range<usize>>) -> Self {
		let range: Range<usize> = range.into();
		Self {
			span: range.into(),
			label: None,
		}
	}

	pub fn point(at: usize, label: impl Into<Cow<'static, str>>) -> Self {
		Self {
			span: miette::SourceSpan::from(at),
			label: Some(label.into()),
		}
	}

	pub fn new(range: impl Into<Range<usize>>, label: impl Into<Cow<'static, str>>) -> Self {
		let range: Range<usize> = range.into();

		Self {
			span: range.into(),
			label: Some(label.into()),
		}
	}

	#[must_use]
	pub fn label(&self) -> Option<&str> {
		self.label.as_deref()
	}
}

impl From<Label> for miette::SourceSpan {
	fn from(value: Label) -> Self {
		value.span
	}
}

impl From<Label> for LabeledSpan {
	fn from(value: Label) -> Self {
		if let Some(message) = value.label {
			Self::at(value.span, message)
		} else {
			Self::underline(value.span)
		}
	}
}

#[derive(Debug)]
pub struct RelatedLabel {
	pub severity: Severity,
	pub message: Cow<'static, str>,
	pub labels: Vec<Label>,
	pub file: Option<Arc<SourceFile>>,
}

impl RelatedLabel {
	pub fn new(severity: Severity, message: impl Into<Cow<'static, str>>) -> Self {
		Self {
			severity,
			message: message.into(),
			labels: Vec::new(),
			file: None,
		}
	}

	pub fn error(message: impl Into<Cow<'static, str>>) -> Self {
		Self::new(Severity::Error, message)
	}

	pub fn warning(message: impl Into<Cow<'static, str>>) -> Self {
		Self::new(Severity::Warning, message)
	}

	pub fn advice(message: impl Into<Cow<'static, str>>) -> Self {
		Self::new(Severity::Advice, message)
	}

	#[must_use]
	pub fn with_source_file(mut self, file: Option<Arc<SourceFile>>) -> Self {
		self.file = file;
		self
	}

	#[must_use]
	pub fn with_labeled_span(
		self,
		span: SourceSpan,
		message: impl Into<Cow<'static, str>>,
	) -> Self {
		let range = span.into_range();
		self.with_label(Label::new(
			(range.start as usize)..(range.end as usize),
			message,
		))
	}

	#[must_use]
	pub fn with_label(mut self, label: Label) -> Self {
		self.labels.push(label);
		self
	}

	#[must_use]
	pub fn with_labels(mut self, labels: impl IntoIterator<Item = Label>) -> Self {
		self.extend(labels);
		self
	}
}

impl Diagnostic for RelatedLabel {
	fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
		None
	}

	fn severity(&self) -> Option<Severity> {
		Some(self.severity)
	}

	fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
		None
	}

	fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
		None
	}

	fn source_code(&self) -> Option<&dyn SourceCode> {
		self.file.as_ref().map(|f| f as &dyn SourceCode)
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
		if self.labels.is_empty() {
			None
		} else {
			Some(Box::new(self.labels.iter().cloned().map(Into::into)))
		}
	}

	fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
		None
	}

	fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
		None
	}
}

impl Display for RelatedLabel {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		f.write_str(&self.message)
	}
}

impl StdError for RelatedLabel {}

impl Extend<Label> for RelatedLabel {
	fn extend<T>(&mut self, iter: T)
	where
		T: IntoIterator<Item = Label>,
	{
		self.labels.extend(iter);
	}
}

#[derive(Debug)]
#[repr(transparent)]
pub struct RelatedError(Report);

impl RelatedError {
	#[must_use]
	pub fn into_report(self) -> Report {
		self.0
	}

	#[must_use]
	pub fn as_diagnostic(&self) -> &dyn Diagnostic {
		self.0.as_ref()
	}

	#[must_use]
	pub const fn new(report: Report) -> Self {
		Self(report)
	}

	pub fn wrap<E>(error: E) -> Self
	where
		E: Diagnostic + Send + Sync + 'static,
	{
		Self::new(Report::new_boxed(Box::new(error)))
	}
}

impl Diagnostic for RelatedError {
	fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
		self.as_diagnostic().code()
	}

	fn severity(&self) -> Option<Severity> {
		self.as_diagnostic().severity()
	}

	fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
		self.as_diagnostic().help()
	}

	fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
		self.as_diagnostic().url()
	}

	fn source_code(&self) -> Option<&dyn SourceCode> {
		self.as_diagnostic().source_code()
	}

	fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
		self.as_diagnostic().labels()
	}

	fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
		self.as_diagnostic().related()
	}

	fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
		self.as_diagnostic().diagnostic_source()
	}
}

impl Display for RelatedError {
	fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
		Display::fmt(&self.0, f)
	}
}

impl StdError for RelatedError {
	fn source(&self) -> Option<&(dyn StdError + 'static)> {
		AsRef::<dyn StdError>::as_ref(&self.0).source()
	}
}

impl From<Report> for RelatedError {
	fn from(value: Report) -> Self {
		Self::new(value)
	}
}
