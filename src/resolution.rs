use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModulePath {
	segments: Vec<String>,
}

impl ModulePath {
	pub fn root() -> Self {
		Self { segments: Vec::new() }
	}

	pub fn from_segments(segments: impl IntoIterator<Item = impl Into<String>>) -> Self {
		Self {
			segments: segments.into_iter().map(Into::into).collect(),
		}
	}

	pub fn child(&self, segment: impl Into<String>) -> Self {
		let mut segments = self.segments.clone();
		segments.push(segment.into());
		Self { segments }
	}

	pub fn segments(&self) -> &[String] {
		&self.segments
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
	Private,
	Public,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
	Module,
	Function,
	Type,
	Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
	pub name: String,
	pub kind: SymbolKind,
	pub visibility: Visibility,
	pub module: ModulePath,
}

impl Symbol {
	pub fn new(name: impl Into<String>, kind: SymbolKind, visibility: Visibility, module: ModulePath) -> Self {
		Self {
			name: name.into(),
			kind,
			visibility,
			module,
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Module {
	pub path: ModulePath,
	pub symbols: BTreeMap<String, Symbol>,
	pub children: BTreeMap<String, ModulePath>,
}

impl Module {
	pub fn new(path: ModulePath) -> Self {
		Self {
			path,
			symbols: BTreeMap::new(),
			children: BTreeMap::new(),
		}
	}
}

#[derive(Debug, Clone, Default)]
pub struct ModuleGraph {
	modules: BTreeMap<ModulePath, Module>,
	prelude: BTreeMap<String, Symbol>,
}

impl ModuleGraph {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn insert_module(&mut self, module: Module) {
		self.modules.insert(module.path.clone(), module);
	}

	pub fn module(&self, path: &ModulePath) -> Option<&Module> {
		self.modules.get(path)
	}

	pub fn module_mut(&mut self, path: &ModulePath) -> Option<&mut Module> {
		self.modules.get_mut(path)
	}

	pub fn insert_child_module(&mut self, parent: &ModulePath, name: impl Into<String>) -> ModulePath {
		let child_name = name.into();
		let child_path = parent.child(child_name.clone());
		self.modules.entry(parent.clone()).or_insert_with(|| Module::new(parent.clone()));
		self.modules.entry(child_path.clone()).or_insert_with(|| Module::new(child_path.clone()));

		if let Some(parent_module) = self.modules.get_mut(parent) {
			parent_module.children.insert(child_name, child_path.clone());
		}

		child_path
	}

	pub fn insert_symbol(&mut self, module_path: &ModulePath, symbol: Symbol) {
		self.modules.entry(module_path.clone()).or_insert_with(|| Module::new(module_path.clone()));
		if let Some(module) = self.modules.get_mut(module_path) {
			module.symbols.insert(symbol.name.clone(), symbol);
		}
	}

	pub fn insert_prelude_symbol(&mut self, symbol: Symbol) {
		self.prelude.insert(symbol.name.clone(), symbol);
	}

	pub fn prelude_symbol(&self, name: &str) -> Option<&Symbol> {
		self.prelude.get(name)
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolutionScope {
	Lexical,
	Module,
	Import,
	Prelude,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSymbol {
	pub symbol: Symbol,
	pub scope: ResolutionScope,
}

#[derive(Debug, Clone)]
pub struct Resolver {
	graph: ModuleGraph,
}

impl Resolver {
	pub fn new(graph: ModuleGraph) -> Self {
		Self { graph }
	}

	pub fn graph(&self) -> &ModuleGraph {
		&self.graph
	}

	pub fn graph_mut(&mut self) -> &mut ModuleGraph {
		&mut self.graph
	}

	pub fn resolve_name(
		&self,
		lexical_scope: &[Symbol],
		module_path: &ModulePath,
		imports: &[ModulePath],
		name: &str,
	) -> Option<ResolvedSymbol> {
		if let Some(symbol) = lexical_scope.iter().rev().find(|symbol| symbol.name == name) {
			return Some(ResolvedSymbol {
				symbol: symbol.clone(),
				scope: ResolutionScope::Lexical,
			});
		}

		if let Some(module) = self.graph.module(module_path) {
			if let Some(symbol) = module.symbols.get(name) {
				return Some(ResolvedSymbol {
					symbol: symbol.clone(),
					scope: ResolutionScope::Module,
				});
			}

			if let Some(child_path) = module.children.get(name) {
				let symbol = Symbol::new(name, SymbolKind::Module, Visibility::Public, child_path.clone());
				return Some(ResolvedSymbol {
					symbol,
					scope: ResolutionScope::Module,
				});
			}
		}

		for import_path in imports {
			if let Some(module) = self.graph.module(import_path) {
				if let Some(symbol) = module.symbols.get(name) {
					if symbol.visibility == Visibility::Public {
						return Some(ResolvedSymbol {
							symbol: symbol.clone(),
							scope: ResolutionScope::Import,
						});
					}
				}
			}
		}

		self.graph.prelude_symbol(name).map(|symbol| ResolvedSymbol {
			symbol: symbol.clone(),
			scope: ResolutionScope::Prelude,
		})
	}
}

pub fn initialize() {}

#[cfg(test)]
mod tests {
	use super::*;

	fn public_symbol(module: &ModulePath, name: &str, kind: SymbolKind) -> Symbol {
		Symbol::new(name, kind, Visibility::Public, module.clone())
	}

	#[test]
	fn module_graph_builds_child_modules() {
		let mut graph = ModuleGraph::new();
		let root = ModulePath::root();
		let child = graph.insert_child_module(&root, "frontend");

		assert_eq!(child.segments(), &["frontend".to_string()]);
		assert!(graph.module(&root).is_some());
		assert!(graph.module(&child).is_some());
	}

	#[test]
	fn resolver_prefers_lexical_scope() {
		let mut graph = ModuleGraph::new();
		let root = ModulePath::root();
		graph.insert_symbol(&root, public_symbol(&root, "value", SymbolKind::Value));
		graph.insert_prelude_symbol(public_symbol(&root, "value", SymbolKind::Type));

		let resolver = Resolver::new(graph);
		let lexical_scope = vec![Symbol::new(
			"value",
			SymbolKind::Function,
			Visibility::Private,
			root.clone(),
		)];

		let resolved = resolver.resolve_name(&lexical_scope, &root, &[], "value").unwrap();
		assert_eq!(resolved.scope, ResolutionScope::Lexical);
		assert_eq!(resolved.symbol.kind, SymbolKind::Function);
	}

	#[test]
	fn resolver_falls_back_to_module_then_import_then_prelude() {
		let mut graph = ModuleGraph::new();
		let root = ModulePath::root();
		let imported = root.child("math");

		graph.insert_symbol(&root, public_symbol(&root, "local", SymbolKind::Value));
		graph.insert_symbol(&imported, public_symbol(&imported, "external", SymbolKind::Function));
		graph.insert_prelude_symbol(public_symbol(&root, "println", SymbolKind::Function));

		let resolver = Resolver::new(graph);

		let local = resolver.resolve_name(&[], &root, &[imported.clone()], "local").unwrap();
		assert_eq!(local.scope, ResolutionScope::Module);

		let imported_symbol = resolver.resolve_name(&[], &root, &[imported.clone()], "external").unwrap();
		assert_eq!(imported_symbol.scope, ResolutionScope::Import);

		let prelude = resolver.resolve_name(&[], &root, &[], "println").unwrap();
		assert_eq!(prelude.scope, ResolutionScope::Prelude);
	}

	#[test]
	fn resolver_rejects_private_imports() {
		let mut graph = ModuleGraph::new();
		let root = ModulePath::root();
		let imported = root.child("secret");

		graph.insert_symbol(
			&imported,
			Symbol::new("hidden", SymbolKind::Value, Visibility::Private, imported.clone()),
		);

		let resolver = Resolver::new(graph);
		assert!(resolver.resolve_name(&[], &root, &[imported], "hidden").is_none());
	}
}
