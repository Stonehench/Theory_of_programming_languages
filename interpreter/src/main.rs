use serde_derive::Deserialize;
use std::{
    collections::HashMap, io::{self, Read}
};


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")] 
enum Expr {
    Application(Vec<Expr>),
    Identifier(String),
    Cond(Vec<Expr>),
    Block(Vec<Expr>),
    Clause(Vec<Expr>),
    Number(i64),
    String(String),
    Parameters(Vec<Expr>),
    Lambda(Vec<Expr>),
    Let(Box<Expr>, Box<Expr>, Box<Expr>),
    Define(Box<Expr>, Box<Expr>),
}



#[derive(Debug, Clone)]
enum ResultValue {
    Number(i64),
    Bool(bool),
    String(String),
    Func(usize, fn(Vec<ResultValue>) -> Result<ResultValue, String>),
    Lambda(Vec<String>, Box<Expr>, Env),
}

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

#[derive(Debug, Clone)]
struct Env {
    vars: HashMap<String, ResultValue>,
    builtins: HashMap<String, ResultValue>,
}

impl Env {
    fn new() -> Self {
        let mut vars = HashMap::new();
        // Initialize the environment with Roman numerals
        vars.insert("x".to_string(), ResultValue::Number(10));
        vars.insert("v".to_string(), ResultValue::Number(5));
        vars.insert("i".to_string(), ResultValue::Number(1));

        // Initialize the environment with built-in functions
        let mut builtins = HashMap::new();
        builtins.insert(
            "add".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Number(a + b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            "sub".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Number(a - b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            "mul".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Number(a * b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
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
        builtins.insert(
            "pow".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Number(a.pow(b as u32))),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
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
        builtins.insert(
            "=".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Bool(a == b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            "<".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Bool(a < b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            ">".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Bool(a > b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            ">=".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Bool(a >= b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            "<=".to_string(),
            ResultValue::Func(2, |args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Number(a), ResultValue::Number(b)) => Ok(ResultValue::Bool(a <= b)),
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );
        builtins.insert(
            "print".to_string(),
            ResultValue::Func(1, |args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                println!("{}", args[0]);
                Ok(ResultValue::Number(0))
            }),
        );
        

        Self { vars, builtins }
    }

    fn get_vars(&self, name: &str) -> Option<ResultValue> {
        self.vars.get(name).cloned()
    }

    fn insert_vars(&mut self, name: String, value: ResultValue) {
        self.vars.insert(name, value);
    }
}

fn eval_expr(expr: Expr, env: &mut Env) -> Result<ResultValue, String> {
    // // backtrace for debugging
    // println!("{:?}", expr);

    match expr {
        Expr::Number(n) => Ok(ResultValue::Number(n)),
        Expr::String(s) => Ok(ResultValue::String(s)),

        Expr::Application(mut args) => {
            let func = eval_expr(args.remove(0), env)?;
            if env.builtins.contains_key(&func.to_string()) {
                return apply_function(env.builtins[&func.to_string()].clone(), args, env);
            }
            apply_function(func, args, env)
        }

        Expr::Identifier(value) => match env.get_vars(&value) {
            Some(val) => Ok(val),
            None => Ok(ResultValue::String(value)),
        },

        Expr::Block(exprs) => {
            let mut result = ResultValue::Number(0);
            for expr in exprs {
                result = eval_expr(expr, env)?;
            }
            Ok(result)
        }

        Expr::Cond(clauses) => {
            for clause in clauses {
                match clause {
                    Expr::Clause(mut clause) => {
                        if clause.len() != 2 {
                            return Err("Each clause must have exactly 2 expressions".to_string());
                        }
                        let cond = eval_expr(clause.remove(0), env)?;
                        if cond.to_string() == "true" {
                            return eval_expr(clause.remove(0), env);
                        } else {
                            clause.remove(0); // Remove the second expression if condition is false
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
            let params = args.remove(0);
            let body_expr = args.remove(0);
            let param_names = if let Expr::Parameters(params) = params {
                params.into_iter().map(|param| {
                    if let Expr::Identifier(name) = param {
                        Ok(name)
                    } else {
                        Err("Invalid parameter".to_string())
                    }
                }).collect::<Result<Vec<_>, _>>()?
            } else {
                return Err("Invalid parameters".to_string());
            };
            Ok(ResultValue::Lambda(param_names, Box::new(body_expr), env.clone()))
        }

        Expr::Let(name, value, body) => {
            let name = if let Expr::Identifier(name) = *name {
                name
            } else {
                return Err("Invalid variable name".to_string());
            };
            let value = eval_expr(*value, env)?;
            env.insert_vars(name, value);
            eval_expr(*body, env)
        }

        Expr::Define(name, value) => {
            let name = if let Expr::Identifier(name) = *name {
                name
            } else {
                return Err("Invalid variable name".to_string());
            };
            let value = eval_expr(*value, env)?;

            env.insert_vars(name, value);
            Ok(ResultValue::Number(0))
        }
    }
}

fn apply_function(f: ResultValue, args: Vec<Expr>, env: &mut Env) -> Result<ResultValue, String> {
    match f {
        ResultValue::Func(args_length, func) => {
            if args.len() != args_length {
                return Err(format!("Expected {} arguments", args_length));
            }

            let arg_values = args
                .into_iter()
                .map(|arg| eval_expr(arg, env))
                .collect::<Result<Vec<_>, _>>()?;

            func(arg_values)
        }
        ResultValue::Lambda(param_names, body, mut lambda_env) => {
            if args.len() != param_names.len() {
                return Err(format!("Expected {} arguments", param_names.len()));
            }

            // Evaluate arguments and extend the environment
            for (param_name, arg) in param_names.into_iter().zip(args.into_iter()) {
                let arg_value = eval_expr(arg, env)?;
                // Lexical scope
                lambda_env.insert_vars(param_name, arg_value);

                // Dynamic scope
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
    // Simulating the environment being initialized
    let mut env = Env::new();

    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read input");

    // println!("{}", input);
    // Parse the input as JSON
    let expr: Expr = serde_json::from_str(&input).expect("JSON was not well-formatted");

    // Evaluate the expression
    match eval_expr(expr, &mut env) {
        Ok(result) => println!("{}", result),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}
