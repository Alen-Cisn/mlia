use crate::parser::Expr;
use inkwell::OptimizationLevel;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::execution_engine::{ExecutionEngine, JitFunction};
use inkwell::module::Module;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::{FunctionValue, IntValue, PointerValue};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;

/// Convenience type alias for the main function.
/// Returns an i64 value representing the program's exit code.
type MainFunc = unsafe extern "C" fn() -> i64;

/// LLVM code generator for the MLIA language.
///
/// This struct manages the LLVM context, module, builder, and execution engine
/// to compile MLIA AST expressions into executable LLVM IR.
pub struct CodeGen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    execution_engine: ExecutionEngine<'ctx>,

    /// Symbol table for variables in the current scope
    variables: HashMap<String, PointerValue<'ctx>>,

    /// Current function being compiled
    current_function: Option<FunctionValue<'ctx>>,

    /// Print function for output operations
    print_function: Option<FunctionValue<'ctx>>,
}

impl<'ctx> CodeGen<'ctx> {
    /// Creates a new CodeGen instance with the given context.
    pub fn new(context: &'ctx Context) -> Result<Self, Box<dyn Error>> {
        let module = context.create_module("mlia_module");
        let execution_engine = module.create_jit_execution_engine(OptimizationLevel::None)?;
        let builder = context.create_builder();

        let mut codegen = CodeGen {
            context,
            module,
            builder,
            execution_engine,
            variables: HashMap::new(),
            current_function: None,
            print_function: None,
        };

        // Declare external print function
        codegen.declare_print_function();

        Ok(codegen)
    }

    /// Declares the external print function for outputting integers.
    /// This links to the C library printf function.
    fn declare_print_function(&mut self) {
        let i32_type = self.context.i32_type();
        let i8_ptr_type = self.context.ptr_type(inkwell::AddressSpace::default());

        // Declare printf function: i32 printf(i8* format, ...)
        let printf_type = i32_type.fn_type(&[i8_ptr_type.into()], true);
        let printf_function = self.module.add_function("printf", printf_type, None);

        self.print_function = Some(printf_function);
    }

    /// Creates a stack allocation for a variable in the entry block of the current function.
    fn create_entry_block_alloca(&self, name: &str) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();
        let entry = self
            .current_function
            .unwrap()
            .get_first_basic_block()
            .unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.i64_type(), name).unwrap()
    }

    /// Builds a load instruction for the given pointer.
    fn build_load(&self, ptr: PointerValue<'ctx>, name: &str) -> IntValue<'ctx> {
        self.builder
            .build_load(self.context.i64_type(), ptr, name)
            .unwrap()
            .into_int_value()
    }

    /// Compiles an expression into an LLVM IntValue.
    fn compile_expr(&mut self, expr: &Expr) -> Result<IntValue<'ctx>, &'static str> {
        match expr {
            Expr::Number(n) => Ok(self.context.i64_type().const_int(*n as u64, true)),

            Expr::Ident(name) => match self.variables.get(name) {
                Some(var) => Ok(self.build_load(*var, name)),
                None => Err("Undefined variable"),
            },

            Expr::Call(func_name, args) => {
                if func_name == "print" && args.len() == 1 {
                    self.compile_print_call(&args[0])
                } else {
                    Err("Unknown function call")
                }
            }

            Expr::Seq(first, second) => {
                // Compile first expression (result is discarded)
                self.compile_expr(first)?;
                // Compile and return second expression
                self.compile_expr(second)
            }

            Expr::Assign(var_name, value) => {
                let val = self.compile_expr(value)?;

                match self.variables.get(var_name) {
                    Some(var) => {
                        self.builder.build_store(*var, val).unwrap();
                        Ok(val)
                    }
                    None => Err("Cannot assign to undefined variable"),
                }
            }

            Expr::Decl(var_name, _params, value, body) => {
                // For now, ignore function parameters (they're empty in our current use case)
                let val = self.compile_expr(value)?;

                // Create stack allocation for the variable
                let alloca = self.create_entry_block_alloca(var_name);

                // Store the initial value
                self.builder.build_store(alloca, val).unwrap();

                // Save old variable binding if it exists
                let old_binding = self.variables.insert(var_name.clone(), alloca);

                // Compile the body with the new variable in scope
                let result = self.compile_expr(body);

                // Restore old binding or remove the variable
                match old_binding {
                    Some(old_var) => {
                        self.variables.insert(var_name.clone(), old_var);
                    }
                    None => {
                        self.variables.remove(var_name);
                    }
                }

                result
            }
        }
    }

    /// Compiles a print function call.
    fn compile_print_call(&mut self, arg: &Expr) -> Result<IntValue<'ctx>, &'static str> {
        let arg_val = self.compile_expr(arg)?;

        // Create format string for printf: "%lld\n"
        let format_str = self
            .builder
            .build_global_string_ptr("%lld\n", "fmt_str")
            .unwrap();

        // Call printf function
        let printf_fn = self.print_function.ok_or("Print function not available")?;
        self.builder
            .build_call(
                printf_fn,
                &[format_str.as_pointer_value().into(), arg_val.into()],
                "printf_call",
            )
            .unwrap();

        // Return the original value
        Ok(arg_val)
    }

    /// Compiles the entire program and returns a JIT-compiled function.
    pub fn compile_program(
        &'_ mut self,
        expr: &Expr,
    ) -> Result<JitFunction<'_, MainFunc>, Box<dyn Error>> {
        // Create main function
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let main_function = self.module.add_function("main", fn_type, None);

        // Create entry basic block
        let entry_block = self.context.append_basic_block(main_function, "entry");
        self.builder.position_at_end(entry_block);

        // Set current function
        self.current_function = Some(main_function);

        // Compile the expression
        let result = self.compile_expr(expr)?;

        // Return the result
        self.builder.build_return(Some(&result)).unwrap();

        // Verify the function
        if main_function.verify(true) {
            // Get the compiled function
            unsafe {
                self.execution_engine
                    .get_function("main")
                    .map_err(|e| format!("Failed to get main function: {}", e).into())
            }
        } else {
            Err("Function verification failed".into())
        }
    }

    /// Compiles the program to an object file and creates an executable.
    pub fn compile_to_executable(
        &mut self,
        expr: &Expr,
        output_path: &str,
    ) -> Result<(), Box<dyn Error>> {
        // Initialize LLVM targets
        Target::initialize_native(&InitializationConfig::default())?;

        // Create main function
        let i64_type = self.context.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let main_function = self.module.add_function("main", fn_type, None);

        // Create entry basic block
        let entry_block = self.context.append_basic_block(main_function, "entry");
        self.builder.position_at_end(entry_block);

        // Set current function
        self.current_function = Some(main_function);

        // Compile the expression
        let result = self.compile_expr(expr)?;

        // Return the result
        self.builder.build_return(Some(&result)).unwrap();

        // Verify the function
        if !main_function.verify(true) {
            return Err("Function verification failed".into());
        }

        // Get the target triple
        let target_triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&target_triple)
            .map_err(|e| format!("Failed to create target from triple: {}", e))?;

        // Create target machine
        let target_machine = target
            .create_target_machine(
                &target_triple,
                "generic",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .ok_or("Failed to create target machine")?;

        // Generate object file
        let obj_path = format!("{}.o", output_path);
        target_machine
            .write_to_file(&self.module, FileType::Object, Path::new(&obj_path))
            .map_err(|e| format!("Failed to write object file: {}", e))?;

        // Link the object file to create an executable
        let link_result = std::process::Command::new("gcc")
            .args(&[&obj_path, "-o", output_path])
            .output()
            .map_err(|e| format!("Failed to run linker: {}", e))?;

        if !link_result.status.success() {
            return Err(format!(
                "Linking failed: {}",
                String::from_utf8_lossy(&link_result.stderr)
            )
            .into());
        }

        // Clean up object file
        fs::remove_file(&obj_path).ok();

        println!("Successfully compiled to executable: {}", output_path);
        Ok(())
    }

    /// Executes the compiled program and returns the exit code.
    pub fn execute_program(&mut self, expr: &Expr) -> Result<i64, Box<dyn Error>> {
        let main_func = self.compile_program(expr)?;

        unsafe {
            let result = main_func.call();
            Ok(result)
        }
    }

    /// Prints the generated LLVM IR to stdout (useful for debugging).
    pub fn print_ir(&self) {
        self.module.print_to_stderr();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Expr;

    #[test]
    fn test_simple_number() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        let expr = Expr::Number(42);
        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 42);
    }

    #[test]
    fn test_variable_declaration() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // decl x <- 5 in x
        let expr = Expr::Decl(
            "x".to_string(),
            vec![],
            Box::new(Expr::Number(5)),
            Box::new(Expr::Ident("x".to_string())),
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 5);
    }

    #[test]
    fn test_sequence() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // 1; 2
        let expr = Expr::Seq(Box::new(Expr::Number(1)), Box::new(Expr::Number(2)));

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 2);
    }
}
