use serde::Deserialize;
use std::collections::HashMap;
use std::io::{self, Read};
use std::rc::Rc;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Expr {
    Identifier(String),                          // Variables
    Number(i64),                                 // Numbers
    Lambda { Parameters: Vec<Expr>, Body: Rc<Expr> }, // Lambda expression
    Application(Vec<Expr>),                      // Function application
    Cond { Clause: Vec<(Expr, Expr)> },          // Conditional expressions
    Block(Vec<Expr>),                            // Block of expressions
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(i64),
    Closure(Vec<String>, Rc<Expr>, Environment), // Closure (captures environment)
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,  // Map of variables to values
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }
}

// Function to evaluate a boolean expression
fn evaluate_bool(
    expr: &Value,
    vars: &HashMap<&str, Value>,
) -> bool {
    if let Some(identifier) = expr.get("Identifier").and_then(|id| id.as_str()) {
        match identifier {
            "true" => true,
            "false" => false,
            _ => panic!("Not a known boolean expression: {}", expr),
        }
    } else if let Some(application) = expr.get("Application") {
        if let Some(operator) = application
            .get(0)
            .and_then(|id| id.get("Identifier"))
            .and_then(|id| id.as_str())
        {
            let left = evaluate_expr(application.get(1).unwrap(), vars);
            if operator == "zero?" {
                return left == 0;
            }
            let right = evaluate_expr(application.get(2).unwrap(), vars);
            match operator {
                "=" => left == right,
                "<" => left < right,
                "<=" => left <= right,
                ">" => left > right,
                ">=" => left >= right,
                _ => panic!("Unknown boolean operator: {}", operator),
            }
        } else {
            panic!("Invalid boolean expression: {:?}", expr);
        }
    } else {
        panic!("Not a known boolean expression: {:?}", expr);
    }
}

// Function to evaluate an expression
fn evaluate_expr(expr: &Value, vars: &HashMap<&str, Value>) -> i64 {
    // Check if the expression is an application
    if let Some(application) = expr.get("Application") {
        if let Some(lambda) = application.get(0).and_then(|id| id.get("Lambda")) {
            // Handle lambda expressions
            if let Some(parameters) = lambda.get(0).and_then(|id| id.get("Parameters")) {
                // Create a new variable map with the parameters
                let mut new_vars = vars.clone();
                for (i, parameter) in parameters.as_array().unwrap().iter().enumerate() {
                    if let Some(identifier) = parameter.get("Identifier").and_then(|id| id.as_str())
                    {
                        new_vars.insert(
                            identifier,
                            application.get(i + 1).unwrap().clone()
                        );
                    }
                }
                // Evaluate the lambda expression
                if let Some(block) = lambda.get(1).and_then(|id| id.get("Block")) {
                    return evaluate_expr(block.get(0).unwrap(), &new_vars);
                } else {
                    panic!("Lambda expression has no block: {:?}", lambda);
                }
            }
        }
        if let Some(identifier) = application
            .get(0)
            .and_then(|id| id.get("Identifier"))
            .and_then(|id| id.as_str())
        {
            // Check if the identifier is a variable
            if let Some(value) = vars.get(identifier) {
                return value.as_i64().expect("Can't return a number"); // Return the value of the variable as i64
            } else {
                // Handle procedures like "add", "sub", etc.
                match identifier {
                    "add" => {
                        // Iterate over the elements and sum them up
                        let mut sum = 0;
                        for item in application.as_array().unwrap().iter().skip(1) {
                            sum += evaluate_expr(item, vars);
                        }
                        return sum;
                    }
                    "sub" => {
                        // Iterate over the elements and subtract them
                        let mut difference = evaluate_expr(application.get(1).unwrap(), vars);
                        for item in application.as_array().unwrap().iter().skip(2) {
                            difference -= evaluate_expr(item, vars);
                        }
                        return difference;
                    }
                    "mul" => {
                        // Iterate over the elements and multiply them
                        let mut product = 1;
                        for item in application.as_array().unwrap().iter().skip(1) {
                            product *= evaluate_expr(item, vars);
                        }
                        return product;
                    }
                    "div" => {
                        // Iterate over the elements and divide them
                        let mut quotient = 1;
                        for item in application.as_array().unwrap().iter().skip(1) {
                            quotient /= evaluate_expr(item, vars);
                        }
                        return quotient;
                    }
                    _ => panic!("Unknown procedure: {}", identifier),
                }
            }
        }
    } else if expr.is_object() {
        // Handle conditional expressions
        if let Some(cond) = expr.get("Cond") {
            for clause in cond.as_array().unwrap() {
                if let Some(clause_array) = clause.get("Clause").and_then(|c| c.as_array()) {
                    if let Some(clause) = clause_array.get(0) {
                        if evaluate_bool(clause, vars) {
                            return evaluate_expr(clause_array.get(1).unwrap(), vars);
                        }
                    }
                }
            }
        }
        // If it's an object with an "Identifier", treat it as a variable reference
        if let Some(identifier) = expr.get("Identifier").and_then(|id| id.as_str()) {
            if let Some(value) = vars.get(identifier) {
                return value.as_i64().expect("Expected a number");
            } 
            else {
                println!("{}", identifier);
                return i64::MIN;
            }
        }
    } else if expr.is_i64() {
        // If it's a direct number, return it
        return expr.as_i64().unwrap();
    }
    panic!("{:?}", expr);
}

fn main() {
    // Variable map where `x`, `v`, and `i` are pre-defined
    let mut vars: HashMap<&str, Value> = HashMap::new();
    vars.insert("x", Value::Number(10.into()));
    vars.insert("v", Value::Number(5.into()));
    vars.insert("i", Value::Number(1.into()));

    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read input");

    // Parse the input as JSON
    let json_input: Value =
        serde_json::from_str(&input).expect("JSON was not well-formatted");

    // Evaluate and print result
    let result = evaluate_expr(&json_input, &vars);
    if result != i64::MIN {
        println!("{}", result);
    }
}