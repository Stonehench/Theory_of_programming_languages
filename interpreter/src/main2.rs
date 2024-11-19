use serde_derive::Deserialize;
use std::{
    collections::HashMap,
    io::{self, Read},
};

// Define the expression types that can be parsed from JSON
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
enum Expr {
    Application(Vec<Expr>),               // Function application
    Identifier(String),                   // Variable or function name
    Cond(Vec<Expr>),                      // Conditional expression
    Block(Vec<Expr>),                     // Block of expressions
    Clause(Vec<Expr>),                    // Clause in a conditional expression
    Number(i64),                          // Integer number
    String(String),                       // String literal
    Parameters(Vec<Expr>),                // Parameters for a lambda function
    Lambda(Vec<Expr>),                    // Lambda function
    Let(Box<Expr>, Box<Expr>, Box<Expr>), // Let binding
    Assignment(Box<Expr>, Box<Expr>),     // Define a variable or function
}

// Define the possible result values of evaluating expressions
#[derive(Debug, Clone)]
enum ResultValue {
    Number(i64),                                                      // Integer number
    Bool(bool),                                                       // Boolean value
    String(String),                                                   // String value
    Func(usize, fn(Vec<ResultValue>) -> Result<ResultValue, String>), // Built-in function
    Lambda(Vec<String>, Box<Expr>, Env),                              // Lambda function
}

// Implement display formatting for ResultValue
impl std::fmt::Display for ResultValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultValue::Number(n) => write!(f, "{}", n),
            ResultValue::Bool(b) => write!(f, "{}", b),
            ResultValue::String(s) => write!(f, "{}", s),
            ResultValue::Func(_, _) => write!(f, "<function>"),
            ResultValue::Lambda(p, b, _) => write!(f, "<lambda {:?} {:?}>", p, b),
        }
    }
}

// Define the environment that holds variables and built-in functions
#[derive(Debug, Clone)]
struct Env {
    vars: HashMap<String, Box<ResultValue>>, // Variables defined in the environment
    builtins: HashMap<String, ResultValue>, // Built-in functions
}

impl Env {
    // Create a new environment with initial variables and built-in functions
    fn new() -> Self {
        let mut vars = HashMap::new();
        // Initialize the environment with Roman numerals
        vars.insert("x".to_string(), Box::new(ResultValue::Number(10)));
        vars.insert("v".to_string(), Box::new(ResultValue::Number(5)));
        vars.insert("i".to_string(), Box::new(ResultValue::Number(1)));

        // Initialize the environment with built-in functions
        let mut builtins = HashMap::new();

        // Built-in function for addition
        builtins.insert(
            "add".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Number(a + b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for subtraction
        builtins.insert(
            "sub".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Number(a - b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for multiplication
        builtins.insert(
            "mul".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Number(a * b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for integer division
        builtins.insert(
            "div".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        if b == 0 {
                            Err("Division by zero".to_string())
                        } else {
                            Ok(ResultValue::Number(a / b))
                        }
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for exponentiation
        builtins.insert(
            "pow".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Number(a.pow(b as u32)))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for checking if a number is zero
        builtins.insert(
            "zero?".to_string(),
            ResultValue::Func(1, |args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Number(n) => Ok(ResultValue::Bool(n == 0)),
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for equality
        builtins.insert(
            "eq".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Bool(a == b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for less than
        builtins.insert(
            "<".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Bool(a < b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for greater than
        builtins.insert(
            ">".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Bool(a > b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for greater than or equal to
        builtins.insert(
            ">=".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Bool(a >= b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for less than or equal to
        builtins.insert(
            "<=".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Bool(a <= b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for printing a statement
        builtins.insert(
            "print".to_string(),
            ResultValue::Func(1, |args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                println!("{}", args[0]);
                Ok(ResultValue::Bool(false))
            }),
        );

        // Built-in function for absolute value
        builtins.insert(
            "abs".to_string(),
            ResultValue::Func(1, |args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Number(n) => Ok(ResultValue::Number(n.abs())),
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for finding the maximum of two numbers
        builtins.insert(
            "max".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Number(a.max(b)))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for finding the minimum of two numbers
        builtins.insert(
            "min".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        Ok(ResultValue::Number(a.min(b)))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for finding the factorial of a number
        builtins.insert(
            "fact".to_string(),
            ResultValue::Func(1, |args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Number(n) => {
                        if n < 0 {
                            return Err("Factorial of a negative number is undefined".to_string());
                        }
                        let mut result = 1;
                        for i in 1..=n {
                            result *= i;
                        }
                        Ok(ResultValue::Number(result))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for taking modular of a number by another number
        builtins.insert(
            "mod".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => {
                        if b == 0 {
                            return Err("Division by zero".to_string());
                        }
                        Ok(ResultValue::Number(a % b))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        Self { vars, builtins }
    }

    // Get a variable from the environment
    fn get_vars(&self, name: &str) -> Option<Box<ResultValue>> {
        self.vars.get(name).cloned()
    }

    // Insert a variable into the environment for let bindings
    fn insert_vars(&mut self, name: String, value: ResultValue) {
        self.vars.insert(name, Box::new(value));
    }
}

// Evaluate an expression in the given environment
fn eval_expr(expr: Expr, env: &mut Env) -> Result<ResultValue, String> {
    // backtrace for debugging
    // println!("{:?}", expr);

    match expr {
        Expr::Number(n) => Ok(ResultValue::Number(n)), // Return the number as is
        Expr::String(s) => Ok(ResultValue::String(s)), // Return the string as is

        Expr::Application(mut args) => {
            // Evaluate the function to be applied
            let func = eval_expr(args.remove(0), env)?;
            // Check if the function is a built-in function
            if env.builtins.contains_key(&func.to_string()) {
                return apply_function(env.builtins[&func.to_string()].clone(), args, env);
            }
            // Apply the function
            apply_function(func, args, env)
        }

        Expr::Identifier(value) => match env.get_vars(&value) {
            Some(val) => Ok(*val),      // Return the value of the variable
            None => Ok(ResultValue::String(value)), // Return the identifier as a string if not found
        },

        Expr::Block(exprs) => {
            // Evaluate each expression in the block and return the result of the last one
            let mut result = ResultValue::Number(0);
            for expr in exprs {
                result = eval_expr(expr, env)?;
            }
            Ok(result)
        }

        Expr::Cond(clauses) => {
            // Evaluate each clause in the conditional expression
            for clause in clauses {
                match clause {
                    Expr::Clause(mut clause) => {
                        if clause.len() != 2 {
                            return Err("Each clause must have exactly 2 expressions".to_string());
                        }
                        // Evaluate the condition
                        let cond = eval_expr(clause.remove(0), env)?;
                        if cond.to_string() == "true" {
                            // If the condition is true, evaluate and return the result of the second expression
                            return eval_expr(clause.remove(0), env);
                        } else {
                            // Remove the second expression if the condition is false
                            clause.remove(0);
                        }
                    }
                    _ => return Err("Invalid clause".to_string()),
                }
            }
            Err("No true clause".to_string())
        }

        Expr::Clause(_) => Err("Invalid clause not wrapped in a cond".to_string()),

        Expr::Parameters(_) => Err("Invalid parameters not wrapped in a lambda".to_string()),

        Expr::Lambda(mut args) => {
            if args.len() != 2 {
                return Err("Lambda must have exactly 2 expressions".to_string());
            }
            // Extract parameters and body of the lambda
            let params = args.remove(0);
            let body_expr = args.remove(0);
            let param_names = if let Expr::Parameters(params) = params {
                params
                    .into_iter()
                    .map(|param| {
                        if let Expr::Identifier(name) = param {
                            Ok(name)
                        } else {
                            Err("Invalid parameter".to_string())
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                return Err("Invalid parameters".to_string());
            };
            // Return the lambda function
            Ok(ResultValue::Lambda(
                param_names,
                Box::new(body_expr),
                env.clone(),
            ))
        }

        Expr::Let(name, value, body) => {
            // Evaluate the value to be bound
            let name = if let Expr::Identifier(name) = *name {
                name
            } else {
                return Err("Invalid variable name".to_string());
            };
            let value = eval_expr(*value, env)?;
            // Insert the variable into the environment
            env.insert_vars(name, value);
            // Evaluate the body with the new variable binding
            eval_expr(*body, env)
        }

        Expr::Assignment(name, value) => {
            // Evaluate the value to be defined
            let name = if let Expr::Identifier(name) = *name {
                name
            } else {
                return Err("Invalid variable name".to_string());
            };
            let value = eval_expr(*value, env)?;
            // Insert the variable into the environment
            // env.insert_vars(name, value.clone());

            if let Some(cell) = env.vars.get_mut(&name) {
                *cell = Box::new(value.clone());
            } else {
                Err("Variable not found".to_string())?;
            }

            Ok(value)
        }
    }
}

// Apply a function to arguments in the given environment
fn apply_function(f: ResultValue, args: Vec<Expr>, env: &mut Env) -> Result<ResultValue, String> {
    match f {
        ResultValue::Func(args_length, func) => {
            // Check if the number of arguments matches the expected length
            if args.len() != args_length {
                return Err(format!("Expected {} arguments", args_length));
            }

            // Evaluate each argument
            let arg_values = args
                .into_iter()
                .map(|arg| eval_expr(arg, env))
                .collect::<Result<Vec<_>, _>>()?;

            // Apply the function to the evaluated arguments
            func(arg_values)
        }
        ResultValue::Lambda(param_names, body, mut lambda_env) => {
            // Check if the number of arguments matches the number of parameters
            if args.len() != param_names.len() {
                return Err(format!("Expected {} arguments", param_names.len()));
            }

            // Evaluate arguments and extend the environment
            for (param_name, arg) in param_names.into_iter().zip(args.into_iter()) {
                let arg_value = eval_expr(arg, env)?;
                // Lexical scope: extend the lambda's environment
                lambda_env.insert_vars(param_name, arg_value);

                // Dynamic scope: extend the current environment
                // env.insert_vars(param_name, arg_value);
            }

            // Evaluate the body of the lambda in the extended environment (lexical scope)
            eval_expr(*body, &mut lambda_env)

            // Evaluate the body of the lambda in the extended environment (dynamic scope)
            // eval_expr(*body, env)
        }
        _ => Err("Not a function".to_string()),
    }
}

fn main() {
    // Initialize the environment
    let mut env = Env::new();

    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read input");

    // Parse the input as JSON
    let expr: Expr = serde_json::from_str(&input).expect("JSON was not well-formatted");

    // Evaluate the expression
    match eval_expr(expr, &mut env) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
