use log::debug;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Mutex;

pub mod function;
pub mod instructions;
pub mod runtime;

pub use function::function_builder::FunctionBuilder;
pub use function::{CallStack, Function, InterpretedFunction};
pub use runtime::InvokeResult;

//TODO Use that macro for deriving debug with custom formatting
#[derive(Debug)]
pub enum RuntimeError {
	InvokeOnNonExistingFunction(String),
	/// Tried to access a register outside of the function's scope
	RegisterOutOfBounds,
	/// wrong number of parameters were passed to some function
	/// (n_expected, n_got)
	WrongNumberOfParameters(usize, usize),
	/// Tried to jump to an unknown label
	BadJumpTarget,
	CastError(String),
	Unimplemented(String),
}

pub struct RuntimeErrorStack {
	pub stack: Rc<CallStack>,
	pub error: RuntimeError,
}

impl RuntimeErrorStack {
	pub fn new(error: RuntimeError, stack: Rc<CallStack>) -> Self {
		Self { error, stack }
	}

	pub fn finalize(&self) -> String {
		let mut out = vec![];

		out.push(format!("Error: {:?}", self.error));
		out.push(self.stack.finalize());

		out.join("\n")
	}
}

impl std::fmt::Debug for RuntimeErrorStack {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		f.debug_struct("RuntimeErrorStack")
			.field("error", &self.error)
			.field("stack", &"<CallStack>")
			.finish()
	}
}

#[derive(Debug)]
pub struct Module {
	//TODO Is this necessary?
	name: String,
	// statics: Vec<String>,
	statics: Mutex<HashMap<String, runtime::Value>>,
	functions: HashMap<String, Function>,
}

impl Module {
	pub fn new(name: String) -> Self {
		Module {
			name,
			statics: Mutex::new(HashMap::new()),
			functions: HashMap::new(),
		}
	}

	pub fn build_ir(&mut self) -> String {
		let mut out = Vec::new();
		out.push(format!("ModuleId = {:?};", self.name));
		out.push(format!(""));

		for s in &*self.statics.lock().unwrap() {
			out.push(format!("static {:?};", s));
		}
		out.push(format!(""));

		for (name, f) in &self.functions {
			out.push(f.build_ir(name));
			out.push(format!(""));
		}

		out.join("\n")
	}

	/// Run a function from outside of the module
	pub fn run(&self, fn_name: &str, params: Vec<runtime::Value>) -> runtime::InvokeResult {
		let call_stack = CallStack::default();
		let cs = Rc::new(CallStack::extend(fn_name.to_owned(), Rc::new(call_stack)));

		self.invoke(fn_name, cs, params)
	}

	/// Used internally to run functions
	pub(self) fn invoke(
		&self,
		fn_name: &str,
		cs: Rc<CallStack>,
		params: Vec<runtime::Value>,
	) -> runtime::InvokeResult {
		//TODO: Keep track of how many times each function gets called

		let fn_ = match self.functions.get(fn_name) {
			None => {
				return InvokeResult::runtime(
					RuntimeError::InvokeOnNonExistingFunction(fn_name.to_owned()),
					cs,
				);
			}
			Some(a) => a,
		};

		debug!("Running function {:?}", fn_name);
		match fn_ {
			Function::Interpreted(f) => f.run_interpreted(params, cs, &self),
			Function::Native(f) => f.0(params, cs, &self),
		}
	}

	/// Get a static variable from the module
	pub fn get_static(&self, static_name: &str) -> Option<runtime::Value> {
		let s = self.statics.lock().unwrap();
		match s.get(static_name) {
			None => None,
			Some(v) => Some(v.clone()),
		}
	}

	/// Set a static variable in the module
	pub fn set_static(&self, static_name: String, new_value: runtime::Value) {
		//TODO Throw error on unknown statics? (useful later when jit compiling?)

		let mut s = self.statics.lock().unwrap();
		s.insert(static_name, new_value);
	}

	/// Add a function to the module
	pub fn add_function(&mut self, name: String, fn_: Function) {
		self.functions.insert(name, fn_);
	}

	/// Add a *global* static variable
	pub fn add_static(&mut self, name: String) {
		//TODO Require a type?
		let mut s = self.statics.lock().unwrap();
		s.insert(name, runtime::Value::Void);
	}
}
