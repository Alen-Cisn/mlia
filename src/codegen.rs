use crate::parser::{Expr, Pattern};
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
                } else if (func_name == "+"
                    || func_name == "-"
                    || func_name == "*"
                    || func_name == "/"
                    || func_name == "%")
                    && args.len() == 2
                {
                    self.compile_binop(func_name, &args[0], &args[1])
                } else if (func_name == "<"
                    || func_name == ">"
                    || func_name == "="
                    || func_name == "!=")
                    && args.len() == 2
                {
                    self.compile_cmp(func_name, &args[0], &args[1])
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

            // Implement While loop codegen (T034-T037)
            Expr::While(condition, body) => self.compile_while(condition, body),

            // Match expressions - pattern matching with exhaustiveness check
            Expr::Match(scrutinee, arms) => self.compile_match(scrutinee, arms),
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

    /// Compiles a binary operation.
    fn compile_binop(
        &mut self,
        op: &str,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<IntValue<'ctx>, &'static str> {
        let lhs_val = self.compile_expr(lhs)?;
        let rhs_val = self.compile_expr(rhs)?;

        let op_result = match op {
            "+" => Some(self.builder.build_int_add(lhs_val, rhs_val, "add")),
            "-" => Some(self.builder.build_int_sub(lhs_val, rhs_val, "sub")),
            "*" => Some(self.builder.build_int_mul(lhs_val, rhs_val, "mul")),
            "/" => Some(self.builder.build_int_signed_div(lhs_val, rhs_val, "div")),
            "%" => Some(self.builder.build_int_signed_rem(lhs_val, rhs_val, "rem")),
            _ => None,
        };

        match op_result {
            Some(result) => match result {
                Ok(val) => Ok(val),
                Err(_) => Err("Failed to build binary operation"),
            },
            None => Err("Invalid operand type"),
        }
    }

    /// Compiles comparison operations into LLVM IR.
    /// Returns 1 for true, 0 for false as i64 values.
    fn compile_cmp(
        &mut self,
        op: &str,
        lhs: &Expr,
        rhs: &Expr,
    ) -> Result<IntValue<'ctx>, &'static str> {
        use inkwell::IntPredicate;

        let lhs_val = self.compile_expr(lhs)?;
        let rhs_val = self.compile_expr(rhs)?;

        let predicate = match op {
            "<" => IntPredicate::SLT, // Signed Less Than
            ">" => IntPredicate::SGT, // Signed Greater Than
            "=" => IntPredicate::EQ,  // Equal
            "!=" => IntPredicate::NE, // Not Equal
            _ => return Err("Invalid comparison operator"),
        };

        let cmp_result = self
            .builder
            .build_int_compare(predicate, lhs_val, rhs_val, "cmp")
            .map_err(|_| "Failed to build comparison")?;

        // Convert i1 (bool) to i64: true -> 1, false -> 0
        self.builder
            .build_int_z_extend(cmp_result, self.context.i64_type(), "cmp_ext")
            .map_err(|_| "Failed to extend comparison result")
    }

    /// Compiles while loops using the standard three-block pattern.
    /// Returns 0 when the loop exits (final condition value).
    fn compile_while(
        &mut self,
        condition: &Expr,
        body: &Expr,
    ) -> Result<IntValue<'ctx>, &'static str> {
        let function = self
            .current_function
            .ok_or("No current function for while loop")?;

        // Create basic blocks
        let loop_header = self.context.append_basic_block(function, "loop_header");
        let loop_body = self.context.append_basic_block(function, "loop_body");
        let loop_exit = self.context.append_basic_block(function, "loop_exit");

        // Branch to header
        self.builder
            .build_unconditional_branch(loop_header)
            .map_err(|_| "Failed to build branch to loop header")?;

        // Header: evaluate condition
        self.builder.position_at_end(loop_header);
        let cond_val = self.compile_expr(condition)?;

        // Convert condition to boolean (non-zero = true, zero = false)
        let cond_bool = self
            .builder
            .build_int_compare(
                inkwell::IntPredicate::NE,
                cond_val,
                self.context.i64_type().const_zero(),
                "loop_cond",
            )
            .map_err(|_| "Failed to build loop condition")?;

        self.builder
            .build_conditional_branch(cond_bool, loop_body, loop_exit)
            .map_err(|_| "Failed to build conditional branch")?;

        // Body: execute loop body
        self.builder.position_at_end(loop_body);
        self.compile_expr(body)?;
        self.builder
            .build_unconditional_branch(loop_header)
            .map_err(|_| "Failed to build branch back to header")?;

        // Exit: continue after loop
        self.builder.position_at_end(loop_exit);

        // Return 0 (final condition value when loop exits)
        Ok(self.context.i64_type().const_zero())
    }

    /// Compiles match expressions with pattern matching.
    /// Requires wildcard pattern for exhaustiveness or returns error.
    /// Returns the value of the matched arm's result expression.
    fn compile_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[(Pattern, Expr)],
    ) -> Result<IntValue<'ctx>, &'static str> {
        // Check for wildcard pattern (exhaustiveness requirement)
        let has_wildcard = arms.iter().any(|(pat, _)| matches!(pat, Pattern::Wildcard));
        if !has_wildcard {
            return Err("Match expression must have wildcard pattern for exhaustiveness");
        }

        let function = self
            .current_function
            .ok_or("No current function for match expression")?;

        // Evaluate scrutinee
        let scrutinee_val = self.compile_expr(scrutinee)?;

        // Create merge block where all arms converge
        let merge_block = self.context.append_basic_block(function, "match_merge");

        // Allocate result variable in entry block
        let saved_insert_point = self.builder.get_insert_block();
        let entry_block = function
            .get_first_basic_block()
            .ok_or("Function has no entry block")?;
        self.builder.position_at_end(entry_block);
        let result_ptr = self.create_entry_block_alloca("match_result");
        if let Some(block) = saved_insert_point {
            self.builder.position_at_end(block);
        }

        // Build comparison chain for each arm
        let mut next_check_block = self.context.append_basic_block(function, "match_check_0");
        self.builder
            .build_unconditional_branch(next_check_block)
            .map_err(|_| "Failed to build branch to first match check")?;

        for (idx, (pattern, result_expr)) in arms.iter().enumerate() {
            self.builder.position_at_end(next_check_block);

            match pattern {
                Pattern::Literal(lit_val) => {
                    // Create blocks for this arm
                    let arm_block = self
                        .context
                        .append_basic_block(function, &format!("match_arm_{}", idx));
                    let next_idx = idx + 1;
                    next_check_block = if next_idx < arms.len() {
                        self.context
                            .append_basic_block(function, &format!("match_check_{}", next_idx))
                    } else {
                        merge_block // Last check goes to merge if no match
                    };

                    // Compare scrutinee with pattern literal
                    let lit_const = self.context.i64_type().const_int(*lit_val as u64, true);
                    let matches = self
                        .builder
                        .build_int_compare(
                            inkwell::IntPredicate::EQ,
                            scrutinee_val,
                            lit_const,
                            &format!("match_cmp_{}", idx),
                        )
                        .map_err(|_| "Failed to build match comparison")?;

                    self.builder
                        .build_conditional_branch(matches, arm_block, next_check_block)
                        .map_err(|_| "Failed to build conditional branch for match arm")?;

                    // Compile arm result expression
                    self.builder.position_at_end(arm_block);
                    let arm_val = self.compile_expr(result_expr)?;
                    self.builder
                        .build_store(result_ptr, arm_val)
                        .map_err(|_| "Failed to store match arm result")?;
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|_| "Failed to build branch to merge block")?;
                }

                Pattern::Wildcard => {
                    // Wildcard always matches - compile result and branch to merge
                    let arm_val = self.compile_expr(result_expr)?;
                    self.builder
                        .build_store(result_ptr, arm_val)
                        .map_err(|_| "Failed to store wildcard result")?;
                    self.builder
                        .build_unconditional_branch(merge_block)
                        .map_err(|_| "Failed to build branch from wildcard to merge")?;
                }
            }
        }

        // Position at merge block and load result
        self.builder.position_at_end(merge_block);
        Ok(self.build_load(result_ptr, "match_result"))
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

    // T011: Test addition operator (US1)
    #[test]
    fn test_addition() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // + 5 3 should equal 8
        let expr = Expr::Call("+".to_string(), vec![Expr::Number(5), Expr::Number(3)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 8, "5 + 3 should equal 8");
    }

    // T012: Test subtraction operator (US1)
    #[test]
    fn test_subtraction() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // - 10 4 should equal 6
        let expr = Expr::Call("-".to_string(), vec![Expr::Number(10), Expr::Number(4)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 6, "10 - 4 should equal 6");
    }

    // T013: Test multiplication operator (US1)
    #[test]
    fn test_multiplication() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // * 6 7 should equal 42
        let expr = Expr::Call("*".to_string(), vec![Expr::Number(6), Expr::Number(7)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 42, "6 * 7 should equal 42");
    }

    // T014: Test division operator (US1)
    #[test]
    fn test_division() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // / 17 5 should equal 3 (integer division)
        let expr = Expr::Call("/".to_string(), vec![Expr::Number(17), Expr::Number(5)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 3, "17 / 5 should equal 3 (integer division)");
    }

    // T015: Test modulo operator (US1)
    #[test]
    fn test_modulo() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // % 17 5 should equal 2
        let expr = Expr::Call("%".to_string(), vec![Expr::Number(17), Expr::Number(5)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 2, "17 % 5 should equal 2");
    }

    // T016: Test negative number handling (US1)
    #[test]
    fn test_negative_numbers() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // + (-5) 3 should equal -2
        let expr = Expr::Call("+".to_string(), vec![Expr::Number(-5), Expr::Number(3)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, -2, "-5 + 3 should equal -2");
    }

    // T020: Test less than operator - true case (US2)
    #[test]
    fn test_less_than_true() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // < 5 10 should equal 1 (true)
        let expr = Expr::Call("<".to_string(), vec![Expr::Number(5), Expr::Number(10)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 1, "5 < 10 should be true (1)");
    }

    // T021: Test less than operator - false case (US2)
    #[test]
    fn test_less_than_false() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // < 10 5 should equal 0 (false)
        let expr = Expr::Call("<".to_string(), vec![Expr::Number(10), Expr::Number(5)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 0, "10 < 5 should be false (0)");
    }

    // T022: Test greater than operator (US2)
    #[test]
    fn test_greater_than() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // > 10 5 should equal 1 (true)
        let expr = Expr::Call(">".to_string(), vec![Expr::Number(10), Expr::Number(5)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 1, "10 > 5 should be true (1)");
    }

    // T023: Test equality operator - true case (US2)
    #[test]
    fn test_equality_true() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // = 7 7 should equal 1 (true)
        let expr = Expr::Call("=".to_string(), vec![Expr::Number(7), Expr::Number(7)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 1, "7 = 7 should be true (1)");
    }

    // T024: Test equality operator - false case (US2)
    #[test]
    fn test_equality_false() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // = 5 10 should equal 0 (false)
        let expr = Expr::Call("=".to_string(), vec![Expr::Number(5), Expr::Number(10)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 0, "5 = 10 should be false (0)");
    }

    // T025: Test not equal operator (US2)
    #[test]
    fn test_not_equal() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // != 5 10 should equal 1 (true)
        let expr = Expr::Call("!=".to_string(), vec![Expr::Number(5), Expr::Number(10)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 1, "5 != 10 should be true (1)");
    }

    // T030: Test while loop countdown (US3)
    #[test]
    fn test_while_loop_countdown() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // decl x <- 3 in while x do x <- - x 1 done
        // Should loop 3 times, decrementing x each time
        let expr = Expr::Decl(
            "x".to_string(),
            vec![],
            Box::new(Expr::Number(3)),
            Box::new(Expr::While(
                Box::new(Expr::Ident("x".to_string())),
                Box::new(Expr::Assign(
                    "x".to_string(),
                    Box::new(Expr::Call(
                        "-".to_string(),
                        vec![Expr::Ident("x".to_string()), Expr::Number(1)],
                    )),
                )),
            )),
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(
            result, 0,
            "While loop should return 0 when condition becomes false"
        );
    }

    // T031: Test while loop with zero iterations (US3)
    #[test]
    fn test_while_loop_zero_iterations() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // while 0 do 42 done
        // Should not execute body at all
        let expr = Expr::While(Box::new(Expr::Number(0)), Box::new(Expr::Number(42)));

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(
            result, 0,
            "While loop with false condition should return 0 immediately"
        );
    }

    // T032: Test while loop with accumulator (US3)
    #[test]
    fn test_while_loop_accumulator() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // decl sum <- 0 in decl i <- 5 in
        // while i do (sum <- + sum i; i <- - i 1) done; sum
        let expr = Expr::Decl(
            "sum".to_string(),
            vec![],
            Box::new(Expr::Number(0)),
            Box::new(Expr::Decl(
                "i".to_string(),
                vec![],
                Box::new(Expr::Number(5)),
                Box::new(Expr::Seq(
                    Box::new(Expr::While(
                        Box::new(Expr::Ident("i".to_string())),
                        Box::new(Expr::Seq(
                            Box::new(Expr::Assign(
                                "sum".to_string(),
                                Box::new(Expr::Call(
                                    "+".to_string(),
                                    vec![
                                        Expr::Ident("sum".to_string()),
                                        Expr::Ident("i".to_string()),
                                    ],
                                )),
                            )),
                            Box::new(Expr::Assign(
                                "i".to_string(),
                                Box::new(Expr::Call(
                                    "-".to_string(),
                                    vec![Expr::Ident("i".to_string()), Expr::Number(1)],
                                )),
                            )),
                        )),
                    )),
                    Box::new(Expr::Ident("sum".to_string())),
                )),
            )),
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 15, "Sum of 5+4+3+2+1 should be 15");
    }

    // T033: Test nested while loops (US3)
    #[test]
    fn test_nested_while_loops() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // decl outer <- 2 in while outer do (
        //   decl inner <- 2 in while inner do inner <- - inner 1 done;
        //   outer <- - outer 1
        // ) done
        let expr = Expr::Decl(
            "outer".to_string(),
            vec![],
            Box::new(Expr::Number(2)),
            Box::new(Expr::While(
                Box::new(Expr::Ident("outer".to_string())),
                Box::new(Expr::Seq(
                    Box::new(Expr::Decl(
                        "inner".to_string(),
                        vec![],
                        Box::new(Expr::Number(2)),
                        Box::new(Expr::While(
                            Box::new(Expr::Ident("inner".to_string())),
                            Box::new(Expr::Assign(
                                "inner".to_string(),
                                Box::new(Expr::Call(
                                    "-".to_string(),
                                    vec![Expr::Ident("inner".to_string()), Expr::Number(1)],
                                )),
                            )),
                        )),
                    )),
                    Box::new(Expr::Assign(
                        "outer".to_string(),
                        Box::new(Expr::Call(
                            "-".to_string(),
                            vec![Expr::Ident("outer".to_string()), Expr::Number(1)],
                        )),
                    )),
                )),
            )),
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 0, "Nested while loops should complete successfully");
    }

    // T038: Test match expression - first pattern matches (US4)
    #[test]
    fn test_match_first_pattern() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // match 1 with | 1 -> 100 | 2 -> 200 | _ -> 300
        let expr = Expr::Match(
            Box::new(Expr::Number(1)),
            vec![
                (Pattern::Literal(1), Expr::Number(100)),
                (Pattern::Literal(2), Expr::Number(200)),
                (Pattern::Wildcard, Expr::Number(300)),
            ],
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 100, "match 1 should return 100");
    }

    // T039: Test match expression - second pattern matches (US4)
    #[test]
    fn test_match_second_pattern() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // match 2 with | 1 -> 100 | 2 -> 200 | _ -> 300
        let expr = Expr::Match(
            Box::new(Expr::Number(2)),
            vec![
                (Pattern::Literal(1), Expr::Number(100)),
                (Pattern::Literal(2), Expr::Number(200)),
                (Pattern::Wildcard, Expr::Number(300)),
            ],
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 200, "match 2 should return 200");
    }

    // T040: Test match expression - wildcard matches (US4)
    #[test]
    fn test_match_wildcard() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // match 5 with | 1 -> 100 | 2 -> 200 | _ -> 300
        let expr = Expr::Match(
            Box::new(Expr::Number(5)),
            vec![
                (Pattern::Literal(1), Expr::Number(100)),
                (Pattern::Literal(2), Expr::Number(200)),
                (Pattern::Wildcard, Expr::Number(300)),
            ],
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 300, "match 5 should return 300 (wildcard)");
    }

    // T041: Test match expression with computed result expressions (US4)
    #[test]
    fn test_match_computed_results() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // match 1 with | 1 -> (+ 10 20) | _ -> 0
        let expr = Expr::Match(
            Box::new(Expr::Number(1)),
            vec![
                (
                    Pattern::Literal(1),
                    Expr::Call("+".to_string(), vec![Expr::Number(10), Expr::Number(20)]),
                ),
                (Pattern::Wildcard, Expr::Number(0)),
            ],
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 30, "match 1 should compute 10 + 20 = 30");
    }

    // T042: Test match as subexpression (US4)
    #[test]
    fn test_match_as_subexpression() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // + (match 2 with | 1 -> 10 | 2 -> 20 | _ -> 30) 5
        let match_expr = Expr::Match(
            Box::new(Expr::Number(2)),
            vec![
                (Pattern::Literal(1), Expr::Number(10)),
                (Pattern::Literal(2), Expr::Number(20)),
                (Pattern::Wildcard, Expr::Number(30)),
            ],
        );

        let expr = Expr::Call("+".to_string(), vec![match_expr, Expr::Number(5)]);

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 25, "20 + 5 should equal 25");
    }

    // T043: Test match with variable in scrutinee (US4)
    #[test]
    fn test_match_with_variable() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // (defvar x 2 in (match x with | 1 -> 100 | 2 -> 200 | _ -> 300))
        let expr = Expr::Decl(
            "x".to_string(),
            vec![],
            Box::new(Expr::Number(2)),
            Box::new(Expr::Match(
                Box::new(Expr::Ident("x".to_string())),
                vec![
                    (Pattern::Literal(1), Expr::Number(100)),
                    (Pattern::Literal(2), Expr::Number(200)),
                    (Pattern::Wildcard, Expr::Number(300)),
                ],
            )),
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 200, "match x (where x=2) should return 200");
    }

    // T044: Test match without wildcard but with all cases covered (US4)
    #[test]
    fn test_match_explicit_patterns() {
        let context = Context::create();
        let mut codegen = CodeGen::new(&context).unwrap();

        // match 0 with | 0 -> 42 | _ -> 0
        let expr = Expr::Match(
            Box::new(Expr::Number(0)),
            vec![
                (Pattern::Literal(0), Expr::Number(42)),
                (Pattern::Wildcard, Expr::Number(0)),
            ],
        );

        let result = codegen.execute_program(&expr).unwrap();
        assert_eq!(result, 42, "match 0 should return 42");
    }
}
