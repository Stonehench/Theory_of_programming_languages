use serde_derive::Deserialize;
use std::{
    cell::RefCell,
    collections::HashMap,
    io::{self, Read},
    rc::Rc,
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
    Number(i64),                                               // Integer number
    Bool(bool),                                                // Boolean value
    String(String),                                            // String value
    Func(fn(Vec<ResultValue>) -> Result<ResultValue, String>), // Built-in function
    Lambda(Vec<String>, Box<Expr>, Env),                       // Lambda function

    Vec(Vec<ResultValue>), // Array for fun
}

// Implement display formatting for ResultValue
impl std::fmt::Display for ResultValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResultValue::Number(n) => write!(f, "{}", n),
            ResultValue::Bool(b) => write!(f, "{}", b),
            ResultValue::String(s) => write!(f, "{}", s),
            ResultValue::Func(_) => write!(f, "<function>"),
            ResultValue::Lambda(p, b, _) => write!(f, "<lambda {:?} {:?}>", p, b),
            ResultValue::Vec(v) => {
                write!(f, "[")?;
                for (i, val) in v.iter().enumerate() {
                    if i == v.len() - 1 {
                        write!(f, "{}", val)?;
                    } else {
                        write!(f, "{}, ", val)?;
                    }
                }
                write!(f, "]")?;
                Ok(())
            }
        }
    
    }
}

// Define the environment that holds variables and built-in functions
#[derive(Debug, Clone)]
struct Env {
    vars: HashMap<String, Rc<RefCell<ResultValue>>>, // Variables defined in the environment
    builtins: HashMap<String, ResultValue>,          // Built-in functions
    parent: Option<Box<Env>>,                        // Parent environment
}

impl Env {
    // Create a new environment with initial variables and built-in functions
    fn new() -> Self {
        let mut vars = HashMap::new();
        // Initialize the environment with Roman numerals
        vars.insert(
            "i".to_string(),
            Rc::new(RefCell::new(ResultValue::Number(1))),
        );
        vars.insert(
            "v".to_string(),
            Rc::new(RefCell::new(ResultValue::Number(5))),
        );
        vars.insert(
            "x".to_string(),
            Rc::new(RefCell::new(ResultValue::Number(10))),
        );

        // Initialize the environment with built-in functions
        let mut builtins = HashMap::new();

        // Built-in function for addition
        builtins.insert(
            "add".to_string(),
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
                for arg in args {
                    print!("{} ", arg);
                }
                println!();

                Ok(ResultValue::Bool(false))
            }),
        );

        // Built-in function for absolute value
        builtins.insert(
            "abs".to_string(),
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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
            ResultValue::Func(|args| {
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

        // Built-in function for waiting for a number of seconds
        builtins.insert(
            "wait".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Number(n) => {
                        std::thread::sleep(std::time::Duration::from_millis(n as u64));
                        Ok(ResultValue::Bool(false))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for creating arrays of integers
        builtins.insert(
            "intArray".to_string(),
            ResultValue::Func(|args| {
                let mut result = Vec::new();
                for arg in args {
                    match arg {
                        ResultValue::Number(n) => result.push(ResultValue::Number(n)),
                        _ => return Err("Invalid argument".to_string()),
                    }
                }
                Ok(ResultValue::Vec(result))
            }),
        );

        // Built-in function for creating arrays of strings
        builtins.insert(
            "stringArray".to_string(),
            ResultValue::Func(|args| {
                let mut result = Vec::new();
                for arg in args {
                    match arg {
                        ResultValue::String(s) => result.push(ResultValue::String(s)),
                        _ => return Err("Invalid argument".to_string()),
                    }
                }
                Ok(ResultValue::Vec(result))
            }),
        );

        // Built-in function for getting the length of an array
        builtins.insert(
            "len".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => Ok(ResultValue::Number(v.len() as i64)),
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting the element at an index in an array
        builtins.insert(
            "get".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Vec(v), ResultValue::Number(i)) => {
                        if i < 0 || i as usize >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        Ok(v[i as usize].clone())
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for setting the element at an index in an array
        builtins.insert(
            "set".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 3 {
                    return Err("Expected exactly 3 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone(), args[2].clone()) {
                    (ResultValue::Vec(mut v), ResultValue::Number(i), value) => {
                        if i < 0 || i as usize >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        v[i as usize] = value;
                        Ok(ResultValue::Vec(v))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for appending an element to an array
        builtins.insert(
            "append".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Vec(mut v), value) => {
                        v.push(value);
                        Ok(ResultValue::Vec(v))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for removing an element at an index in an array
        builtins.insert(
            "remove".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Vec(mut v), ResultValue::Number(i)) => {
                        if i < 0 || i as usize >= v.len() {
                            return Err("Index out of bounds".to_string());
                        }
                        v.remove(i as usize);
                        Ok(ResultValue::Vec(v))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for reversing an array
        builtins.insert(
            "rev".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(mut v) => {
                        v.reverse();
                        Ok(ResultValue::Vec(v))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for sorting an array
        builtins.insert(
            "sort".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(mut v) => {
                        v.sort_by(|a, b| match (a, b) {
                            (ResultValue::Number(a), ResultValue::Number(b)) => a.cmp(b),
                            _ => panic!("Invalid argument"),
                        });
                        Ok(ResultValue::Vec(v))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for checking if an array is empty
        builtins.insert(
            "empty?".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => Ok(ResultValue::Bool(v.is_empty())),
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting the head of an array
        builtins.insert(
            "head".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        if v.is_empty() {
                            return Err("Array is empty".to_string());
                        }
                        Ok(v[0].clone())
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting the tail of an array
        builtins.insert(
            "tail".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        if v.is_empty() {
                            return Err("Array is empty".to_string());
                        }
                        Ok(ResultValue::Vec(v[1..].to_vec()))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting the last element of an array
        builtins.insert(
            "last".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        if v.is_empty() {
                            return Err("Array is empty".to_string());
                        }
                        Ok(v[v.len() - 1].clone())
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for applying a function to each element of an array
        builtins.insert(
            "map".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Lambda(params, body, lambda_env), ResultValue::Vec(v)) => {
                        let mut result = Vec::new();
                        for arg in v {
                            let mut new_env = Env::new_with_parent(lambda_env.clone());
                            new_env.insert_vars(params[0].clone(), arg);
                            result.push(eval_expr(*body.clone(), &mut new_env)?);
                        }
                        Ok(ResultValue::Vec(result))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for filtering an array
        builtins.insert(
            "filter".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 2 {
                    return Err("Expected exactly 2 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone()) {
                    (ResultValue::Lambda(params, body, lambda_env), ResultValue::Vec(v)) => {
                        let mut result = Vec::new();
                        for arg in v {
                            let mut new_env = Env::new_with_parent(lambda_env.clone());
                            new_env.insert_vars(params[0].clone(), arg.clone());
                            if eval_expr(*body.clone(), &mut new_env)?.to_string() == "true" {
                                result.push(arg);
                            }
                        }
                        Ok(ResultValue::Vec(result))
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for folding an array
        builtins.insert(
            "fold".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 3 {
                    return Err("Expected exactly 3 arguments".to_string());
                }

                match (args[0].clone(), args[1].clone(), args[2].clone()) {
                    (ResultValue::Lambda(params, body, lambda_env), acc, ResultValue::Vec(v)) => {
                        let mut result = acc;
                        for arg in v {
                            let mut new_env = Env::new_with_parent(lambda_env.clone());
                            new_env.insert_vars(params[0].clone(), result.clone());
                            new_env.insert_vars(params[1].clone(), arg);
                            result = eval_expr(*body.clone(), &mut new_env)?;
                        }
                        Ok(result)
                    }
                    _ => Err("Invalid arguments".to_string()),
                }
            }),
        );

        // Built-in function for summing an array
        builtins.insert(
            "sum".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        let mut result = 0;
                        for arg in v {
                            match arg {
                                ResultValue::Number(n) => result += n,
                                _ => return Err("Invalid argument".to_string()),
                            }
                        }
                        Ok(ResultValue::Number(result))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for finding the product of an array
        builtins.insert(
            "product".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        let mut result = 1;
                        for arg in v {
                            match arg {
                                ResultValue::Number(n) => result *= n,
                                _ => return Err("Invalid argument".to_string()),
                            }
                        }
                        Ok(ResultValue::Number(result))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting median of an array
        builtins.insert(
            "median".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(mut v) => {
                        v.sort_by(|a, b| match (a, b) {
                            (ResultValue::Number(a), ResultValue::Number(b)) => a.cmp(b),
                            _ => panic!("Invalid argument"),
                        });
                        let len = v.len();
                        if len % 2 == 0 {
                            let mid = len / 2;
                            match (v[mid - 1].clone(), v[mid].clone()) {
                                (ResultValue::Number(a), ResultValue::Number(b)) => {
                                    Ok(ResultValue::Number((a + b) / 2))
                                }
                                _ => Err("Invalid argument".to_string()),
                            }
                        } else {
                            match v[len / 2].clone() {
                                ResultValue::Number(n) => Ok(ResultValue::Number(n)),
                                _ => Err("Invalid argument".to_string()),
                            }
                        }
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting mean of an array
        builtins.insert(
            "mean".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        let mut sum = 0;
                        let mut count = 0;
                        for arg in v {
                            match arg {
                                ResultValue::Number(n) => {
                                    sum += n;
                                    count += 1;
                                }
                                _ => return Err("Invalid argument".to_string()),
                            }
                        }
                        if count == 0 {
                            return Err("Array is empty".to_string());
                        }
                        Ok(ResultValue::Number(sum / count as i64))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );
        
        // Built-in function for getting max value of an array
        builtins.insert(
            "maxArray".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        let mut max = std::i64::MIN;
                        for arg in v {
                            match arg {
                                ResultValue::Number(n) => {
                                    if n > max {
                                        max = n;
                                    }
                                }
                                _ => return Err("Invalid argument".to_string()),
                            }
                        }
                        if max == std::i64::MIN {
                            return Err("Array is empty".to_string());
                        }
                        Ok(ResultValue::Number(max))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        // Built-in function for getting min value of an array
        builtins.insert(
            "minArray".to_string(),
            ResultValue::Func(|args| {
                if args.len() != 1 {
                    return Err("Expected exactly 1 argument".to_string());
                }

                match args[0].clone() {
                    ResultValue::Vec(v) => {
                        let mut min = std::i64::MAX;
                        for arg in v {
                            match arg {
                                ResultValue::Number(n) => {
                                    if n < min {
                                        min = n;
                                    }
                                }
                                _ => return Err("Invalid argument".to_string()),
                            }
                        }
                        if min == std::i64::MAX {
                            return Err("Array is empty".to_string());
                        }
                        Ok(ResultValue::Number(min))
                    }
                    _ => Err("Invalid argument".to_string()),
                }
            }),
        );

        Self {
            vars,
            builtins,
            parent: None,
        }
    }

    // Create a new environment with a parent
    fn new_with_parent(parent: Env) -> Self {
        Self {
            vars: HashMap::new(),
            builtins: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    // Get a variable from the environment
    fn get_vars(&self, name: &str) -> Option<Rc<RefCell<ResultValue>>> {
        self.vars.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.get_vars(name))
        })
    }

    // Insert a variable into the environment for let bindings
    fn insert_vars(&mut self, name: String, value: ResultValue) {
        self.vars.insert(name, Rc::new(RefCell::new(value)));
    }

    // Update a variable in the environment where it was originally defined (dereferencing the Box)
    fn update_vars_deref(&mut self, name: &str, value: ResultValue) -> Result<(), String> {
        if let Some(cell) = self.vars.get_mut(name) {
            *cell.borrow_mut() = value;
            Ok(())
        } else if let Some(ref mut parent) = self.parent {
            parent.update_vars_deref(name, value)
        } else {
            Err("Variable not found".to_string())
        }
    }

    fn get_builtins(&self, name: &str) -> Option<ResultValue> {
        self.builtins.get(name).cloned().or_else(|| {
            self.parent
                .as_ref()
                .and_then(|parent| parent.get_builtins(name))
        })
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
            if let Some(built_in_func) = env.get_builtins(&func.to_string()) {
                return apply_function(built_in_func, args, env);
            }
            // Apply the function
            apply_function(func, args, env)
        }

        Expr::Identifier(value) => match env.get_vars(&value) {
            Some(val) => Ok(val.borrow().clone()), // Return the value of the variable
            None => Ok(ResultValue::String(value)), // Return the identifier as a string if not found
        },

        Expr::Block(exprs) => {
            // Create a new environment with the current environment as the parent
            let mut block_env = Env::new_with_parent(env.clone());
            // Evaluate each expression in the block and return the result of the last one
            let mut result = ResultValue::Bool(false);
            for expr in exprs {
                result = eval_expr(expr, &mut block_env)?;
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
            // Update the variable in the environment where it was originally defined (dereferencing the Box)
            env.update_vars_deref(&name, value.clone())?;
            Ok(value)
        }
    }
}

// Apply a function to arguments in the given environment
fn apply_function(f: ResultValue, args: Vec<Expr>, env: &mut Env) -> Result<ResultValue, String> {
    match f {
        ResultValue::Func(func) => {
            // Evaluate each argument
            let arg_values = args
                .into_iter()
                .map(|arg| eval_expr(arg, env))
                .collect::<Result<Vec<_>, _>>()?;

            // Apply the function to the evaluated arguments
            func(arg_values)
        }
        ResultValue::Lambda(param_names, body, lambda_env) => {
            // Check if the number of arguments matches the number of parameters
            if args.len() != param_names.len() {
                return Err(format!("Expected {} arguments", param_names.len()));
            }

            // Create a new environment with the lambda's environment as the parent
            let mut new_env = Env::new_with_parent(lambda_env);

            // Evaluate arguments and extend the environment
            for (param_name, arg) in param_names.into_iter().zip(args.into_iter()) {
                let arg_value = eval_expr(arg, env)?;
                new_env.insert_vars(param_name, arg_value);
            }

            // Evaluate the body of the lambda in the new environment
            eval_expr(*body, &mut new_env)
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
