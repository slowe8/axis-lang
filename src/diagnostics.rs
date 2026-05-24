use std::fmt;

pub fn initialize() {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
	Error,
	Warning,
	Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
	pub severity: Severity,
	pub code: String,
	pub message: String,
	pub note: Option<String>,
}

impl Diagnostic {
	pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
		Self {
			severity: Severity::Error,
			code: code.into(),
			message: message.into(),
			note: None,
		}
	}

	pub fn warning(code: impl Into<String>, message: impl Into<String>) -> Self {
		Self {
			severity: Severity::Warning,
			code: code.into(),
			message: message.into(),
			note: None,
		}
	}

	pub fn info(code: impl Into<String>, message: impl Into<String>) -> Self {
		Self {
			severity: Severity::Info,
			code: code.into(),
			message: message.into(),
			note: None,
		}
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Diagnostics {
	entries: Vec<Diagnostic>,
}

impl Diagnostics {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push(&mut self, diagnostic: Diagnostic) {
		self.entries.push(diagnostic);
	}

	pub fn error(&mut self, code: impl Into<String>, message: impl Into<String>) {
		self.push(Diagnostic::error(code, message));
	}

	pub fn warning(&mut self, code: impl Into<String>, message: impl Into<String>) {
		self.push(Diagnostic::warning(code, message));
	}

	pub fn info(&mut self, code: impl Into<String>, message: impl Into<String>) {
		self.push(Diagnostic::info(code, message));
	}

	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	pub fn has_errors(&self) -> bool {
		self.entries.iter().any(|entry| entry.severity == Severity::Error)
	}

	pub fn entries(&self) -> &[Diagnostic] {
		&self.entries
	}

	pub fn extend<I>(&mut self, diagnostics: I)
	where
		I: IntoIterator<Item = Diagnostic>,
	{
		self.entries.extend(diagnostics);
	}
}

impl fmt::Display for Diagnostic {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match &self.note {
			Some(note) => write!(f, "[{:#?}] {}: {} ({})", self.severity, self.code, self.message, note),
			None => write!(f, "[{:#?}] {}: {}", self.severity, self.code, self.message),
		}
	}
}
