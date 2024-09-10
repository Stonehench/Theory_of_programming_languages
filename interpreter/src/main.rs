use std::collections::HashMap;
use std::io::{self, Read};

// Function to evaluate an expression
fn evaluate(expr: &str, vars: &HashMap<&str, i64>) -> i64 {
    let binding = expr.replace("(", " ( ").replace(")", " ) ");
    let tokens: Vec<&str> = binding.split_whitespace().collect();

    let mut stack = vec![];
    let mut i = 0;

    fn parse(tokens: &[&str], i: &mut usize, vars: &HashMap<&str, i64>) -> i64 {
        if tokens[*i] == "(" {
            *i += 1; // Skip '('
            let op = tokens[*i];
            *i += 1; // Move to first argument

            let left = parse(tokens, i, vars);
            let right = parse(tokens, i, vars);
            *i += 1; // Skip ')'

            match op {
                "add" => left + right,
                "sub" => left - right,
                _ => panic!("Unknown operation: {}", op),
            }
        } else if let Ok(num) = tokens[*i].parse::<i64>() {
            let result = num;
            *i += 1;
            result
        } else {
            // It's a variable like `x`
            let var = tokens[*i];
            *i += 1;
            *vars
                .get(var)
                .expect(&format!("Undefined variable: {}", var))
        }
    }

    while i < tokens.len() {
        stack.push(parse(&tokens, &mut i, vars));
    }

    stack.pop().unwrap()
}

fn main() {
    // Variable map where `x` is set to 10
    let mut vars = HashMap::new();
    vars.insert("x", 10);
    vars.insert("v", 5);
    vars.insert("i", 1);

    // Read input from stdin
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read input");

    // Evaluate and print result
    if input.starts_with("\"") {
        print!("{}", input);
    } else {
        let result = evaluate(&input.trim(), &vars);
        println!("{}", result);
    }
}
