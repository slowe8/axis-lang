use std::collections::BTreeMap;

use crate::resolution::ModulePath;

pub fn initialize() {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
	Immutable,
	Mutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ownership {
	Owned,
	SharedBorrow,
	MutableBorrow,
	Copy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
	Local,
	Parameter,
	Field,
	Global,
	Import,
	Temporary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
	Binding { name: String },
	Field { base: Box<Place>, name: String },
	Index { base: Box<Place> },
	Temporary,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
	pub name: String,
	pub ty: TypeId,
	pub mutability: Mutability,
	pub ownership: Ownership,
	pub kind: BindingKind,
	pub module: ModulePath,
	pub place: Place,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeFrame {
	pub bindings: BTreeMap<String, Binding>,
	pub type_bindings: BTreeMap<String, TypeId>,
}

impl ScopeFrame {
	pub fn new() -> Self {
		Self {
			bindings: BTreeMap::new(),
			type_bindings: BTreeMap::new(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimitiveType {
	Bool,
	Int,
	Float,
	String,
	Char,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NumericType {
	Scalar { element: TypeId },
	Vector { element: TypeId, length: usize },
	Matrix { element: TypeId, rows: usize, cols: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterType {
	pub name: Option<String>,
	pub ty: TypeId,
	pub mutability: Mutability,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionType {
	pub parameters: Vec<ParameterType>,
	pub return_type: TypeId,
	pub captures: Vec<TypeId>,
	pub is_task: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenericType {
	pub name: String,
	pub bounds: Vec<TypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
	Error,
	Never,
	Unit,
	Primitive(PrimitiveType),
	Named {
		path: Vec<String>,
		arguments: Vec<TypeId>,
	},
	Reference {
		mutable: bool,
		target: TypeId,
	},
	Tuple(Vec<TypeId>),
	Array {
		element: TypeId,
		length: usize,
	},
	Function(FunctionType),
	Task(FunctionType),
	Generic(GenericType),
	Numeric(NumericType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeStore {
	types: Vec<Type>,
	named_types: BTreeMap<String, TypeId>,
}

impl TypeStore {
	pub fn new() -> Self {
		let mut store = Self {
			types: Vec::new(),
			named_types: BTreeMap::new(),
		};
		store.bootstrap_builtins();
		store
	}

	pub fn intern(&mut self, ty: Type) -> TypeId {
		if let Some((index, _)) = self.types.iter().enumerate().find(|(_, existing)| **existing == ty) {
			return TypeId(index);
		}

		let id = TypeId(self.types.len());
		self.types.push(ty);
		id
	}

	pub fn get(&self, id: TypeId) -> Option<&Type> {
		self.types.get(id.0)
	}

	pub fn register_named(&mut self, name: impl Into<String>, ty: Type) -> TypeId {
		let name = name.into();
		let id = self.intern(ty);
		self.named_types.insert(name, id);
		id
	}

	pub fn named(&self, name: &str) -> Option<TypeId> {
		self.named_types.get(name).copied()
	}

	pub fn builtin(&self, name: &str) -> Option<&Type> {
		self.named(name).and_then(|id| self.get(id))
	}

	pub fn len(&self) -> usize {
		self.types.len()
	}

	pub fn is_empty(&self) -> bool {
		self.types.is_empty()
	}

	fn bootstrap_builtins(&mut self) {
		self.register_named("unit", Type::Unit);
		self.register_named("never", Type::Never);
		self.register_named("bool", Type::Primitive(PrimitiveType::Bool));
		self.register_named("int", Type::Primitive(PrimitiveType::Int));
		self.register_named("float", Type::Primitive(PrimitiveType::Float));
		self.register_named("string", Type::Primitive(PrimitiveType::String));
		self.register_named("char", Type::Primitive(PrimitiveType::Char));
	}
}

impl Default for TypeStore {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionContext {
	pub name: String,
	pub module: ModulePath,
	pub signature: FunctionType,
	pub generics: Vec<GenericType>,
	pub is_public: bool,
	pub is_task: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEnvironment {
	pub module: ModulePath,
	pub scopes: Vec<ScopeFrame>,
	pub type_names: BTreeMap<String, TypeId>,
	pub current_function: Option<FunctionContext>,
	pub expected_return: Option<TypeId>,
	pub expected_error: Option<TypeId>,
	pub in_loop_depth: usize,
	pub in_task_context: bool,
	pub allow_mutation: bool,
	pub allow_await: bool,
}

impl TypeEnvironment {
	pub fn new(module: ModulePath) -> Self {
		Self {
			module,
			scopes: vec![ScopeFrame::new()],
			type_names: BTreeMap::new(),
			current_function: None,
			expected_return: None,
			expected_error: None,
			in_loop_depth: 0,
			in_task_context: false,
			allow_mutation: false,
			allow_await: false,
		}
	}

	pub fn push_scope(&mut self) {
		self.scopes.push(ScopeFrame::new());
	}

	pub fn pop_scope(&mut self) -> Option<ScopeFrame> {
		if self.scopes.len() > 1 {
			self.scopes.pop()
		} else {
			None
		}
	}

	pub fn current_scope(&self) -> Option<&ScopeFrame> {
		self.scopes.last()
	}

	pub fn current_scope_mut(&mut self) -> Option<&mut ScopeFrame> {
		self.scopes.last_mut()
	}

	pub fn declare_binding(&mut self, binding: Binding) -> Option<Binding> {
		self.current_scope_mut()
			.and_then(|scope| scope.bindings.insert(binding.name.clone(), binding))
	}

	pub fn declare_type(&mut self, name: impl Into<String>, ty: TypeId) -> Option<TypeId> {
		let name = name.into();
		self.type_names.insert(name, ty)
	}

	pub fn declare_local_type(&mut self, name: impl Into<String>, ty: TypeId) -> Option<TypeId> {
		let name = name.into();
		self.current_scope_mut()
			.and_then(|scope| scope.type_bindings.insert(name, ty))
	}

	pub fn lookup_binding(&self, name: &str) -> Option<&Binding> {
		for scope in self.scopes.iter().rev() {
			if let Some(binding) = scope.bindings.get(name) {
				return Some(binding);
			}
		}

		None
	}

	pub fn lookup_type(&self, name: &str) -> Option<TypeId> {
		for scope in self.scopes.iter().rev() {
			if let Some(ty) = scope.type_bindings.get(name) {
				return Some(*ty);
			}
		}

		self.type_names.get(name).copied()
	}
}

impl Default for TypeEnvironment {
	fn default() -> Self {
		Self::new(ModulePath::root())
	}
}

pub fn initialize_environment(module: ModulePath) -> TypeEnvironment {
	TypeEnvironment::new(module)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn sample_module() -> ModulePath {
		ModulePath::from_segments(["frontend", "types"])
	}

	#[test]
	fn type_store_bootstraps_builtin_types() {
		let store = TypeStore::new();

		assert!(matches!(store.builtin("unit"), Some(Type::Unit)));
		assert!(matches!(store.builtin("bool"), Some(Type::Primitive(PrimitiveType::Bool))));
	}

	#[test]
	fn type_store_deduplicates_interned_types() {
		let mut store = TypeStore::new();
		let first = store.intern(Type::Tuple(vec![TypeId(0), TypeId(1)]));
		let second = store.intern(Type::Tuple(vec![TypeId(0), TypeId(1)]));

		assert_eq!(first, second);
		assert_eq!(store.get(first), Some(&Type::Tuple(vec![TypeId(0), TypeId(1)])));
	}

	#[test]
	fn environment_tracks_binding_lookup_by_scope() {
		let mut env = TypeEnvironment::new(sample_module());
		let value_type = TypeId(7);
		let outer = Binding {
			name: "value".to_string(),
			ty: value_type,
			mutability: Mutability::Immutable,
			ownership: Ownership::Owned,
			kind: BindingKind::Local,
			module: env.module.clone(),
			place: Place::Binding {
				name: "value".to_string(),
			},
		};

		env.declare_binding(outer.clone());
		assert_eq!(env.lookup_binding("value"), Some(&outer));

		env.push_scope();
		let inner = Binding {
			name: "value".to_string(),
			ty: TypeId(11),
			mutability: Mutability::Mutable,
			ownership: Ownership::Copy,
			kind: BindingKind::Parameter,
			module: env.module.clone(),
			place: Place::Temporary,
		};

		env.declare_binding(inner.clone());
		assert_eq!(env.lookup_binding("value"), Some(&inner));
		env.pop_scope();
		assert_eq!(env.lookup_binding("value"), Some(&outer));
	}

	#[test]
	fn environment_tracks_type_names_and_function_context() {
		let mut env = TypeEnvironment::new(sample_module());
		let ty = TypeId(3);
		env.declare_type("Point", ty);

		let signature = FunctionType {
			parameters: vec![ParameterType {
				name: Some("input".to_string()),
				ty,
				mutability: Mutability::Immutable,
			}],
			return_type: TypeId(0),
			captures: vec![TypeId(1)],
			is_task: true,
		};

		env.current_function = Some(FunctionContext {
			name: "compute".to_string(),
			module: env.module.clone(),
			signature: signature.clone(),
			generics: vec![GenericType {
				name: "T".to_string(),
				bounds: vec![ty],
			}],
			is_public: true,
			is_task: true,
		});

		assert_eq!(env.lookup_type("Point"), Some(ty));
		assert_eq!(env.current_function.as_ref().map(|function| &function.signature), Some(&signature));
	}

	#[test]
	fn numeric_and_reference_shapes_are_representable() {
		let mut store = TypeStore::new();
		let float_ty = store.named("float").expect("float builtin");

		let vector = store.intern(Type::Numeric(NumericType::Vector {
			element: float_ty,
			length: 3,
		}));
		let reference = store.intern(Type::Reference {
			mutable: false,
			target: vector,
		});

		assert!(matches!(store.get(vector), Some(Type::Numeric(NumericType::Vector { length: 3, .. }))));
		assert!(matches!(store.get(reference), Some(Type::Reference { mutable: false, .. })));
	}
}
