use crate::ast::{Expression, Program, Statement};
use crate::environment::Environment;
use crate::object::Object;
use crate::token::Token;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

pub fn eval_program(program: &Program, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;

    for statement in &program.statements {
        result = eval_statement(statement, Rc::clone(&env));

        if let Object::ReturnValue(value) = result {
            return *value;
        }
        if let Object::Error(_) = result {
            return result;
        }
    }

    result
}

pub fn eval_statement(statement: &Statement, env: Rc<RefCell<Environment>>) -> Object {
    match statement {
        Statement::Expression(expr) => eval_expression(expr, env),
        Statement::ClassStatement { name, body } => {
            let class_env = Rc::new(RefCell::new(Environment::new()));
            eval_statement(body, Rc::clone(&class_env));
            let class_obj = Object::Class {
                name: name.clone(),
                environment: class_env,
            };
            env.borrow_mut().set(name.clone(), class_obj.clone());
            class_obj
        }
        Statement::Return(expr) => {
            let val = eval_expression(expr, env);
            if is_error(&val) {
                return val;
            }
            Object::ReturnValue(Box::new(val))
        }
        Statement::Let { name, value } => {
            let val = eval_expression(value, Rc::clone(&env));
            if is_error(&val) {
                return val;
            }
            match name {
                Expression::Identifier(id) => {
                    env.borrow_mut().set(id.clone(), val);
                }
                Expression::MemberExpression { left, property } => {
                    let left_obj = eval_expression(left, Rc::clone(&env));
                    if let Object::Instance { instance_env, .. } = left_obj {
                        instance_env.borrow_mut().set(property.clone(), val);
                    } else {
                        return Object::Error(format!("Cannot assign property to {:?}", left_obj));
                    }
                }
                _ => return Object::Error(format!("Invalid left-hand side of let statement: {:?}", name)),
            }
            Object::Null
        }
        Statement::Block(statements) => eval_block_statement(statements, env),
    }
}

pub fn eval_block_statement(statements: &Vec<Statement>, env: Rc<RefCell<Environment>>) -> Object {
    let mut result = Object::Null;

    for statement in statements {
        result = eval_statement(statement, Rc::clone(&env));

        if matches!(result, Object::ReturnValue(_)) || matches!(result, Object::Error(_)) {
            return result; // Don't unwrap ReturnValue yet, let the outer function handle it
        }
    }

    result
}

pub fn eval_expression(expression: &Expression, env: Rc<RefCell<Environment>>) -> Object {
    match expression {
        Expression::Integer(val) => Object::Integer(*val),
        Expression::Boolean(val) => Object::Boolean(*val),
        Expression::String(val) => Object::String(val.clone()),
        Expression::Identifier(name) => eval_identifier(name, env),
        Expression::Prefix { operator, right } => {
            let right_evaluated = eval_expression(right, env);
            if is_error(&right_evaluated) {
                return right_evaluated;
            }
            eval_prefix_expression(operator, right_evaluated)
        }
        Expression::Infix { left, operator, right } => {
            let left_evaluated = eval_expression(left, Rc::clone(&env));
            if is_error(&left_evaluated) {
                return left_evaluated;
            }
            let right_evaluated = eval_expression(right, env);
            if is_error(&right_evaluated) {
                return right_evaluated;
            }
            eval_infix_expression(operator, left_evaluated, right_evaluated)
        }
        Expression::If { condition, consequence, alternative } => {
            let condition_val = eval_expression(condition, Rc::clone(&env));
            if is_error(&condition_val) {
                return condition_val;
            }
            if is_truthy(&condition_val) {
                eval_statement(consequence, env)
            } else if let Some(alt) = alternative {
                eval_statement(alt, env)
            } else {
                Object::Null
            }
        }
        Expression::Function { parameters, body } => {
            Object::Function {
                parameters: parameters.clone(),
                body: body.clone(),
            }
        }
        Expression::ArrayLiteral(elements) => {
            let mut eval_elements = vec![];
            for e in elements {
                let evaluated = eval_expression(e, Rc::clone(&env));
                if is_error(&evaluated) {
                    return evaluated;
                }
                eval_elements.push(evaluated);
            }
            Object::Array(eval_elements)
        }
        Expression::IndexExpression { left, index } => {
            let left_evaluated = eval_expression(left, Rc::clone(&env));
            if is_error(&left_evaluated) {
                return left_evaluated;
            }
            let index_evaluated = eval_expression(index, env);
            if is_error(&index_evaluated) {
                return index_evaluated;
            }
            eval_index_expression(left_evaluated, index_evaluated)
        }
        Expression::HashLiteral(pairs) => {
            let mut hash = HashMap::new();
            for (key_node, val_node) in pairs {
                let key = eval_expression(key_node, Rc::clone(&env));
                if is_error(&key) {
                    return key;
                }
                let hash_key = match key.get_hash_key() {
                    Ok(hk) => hk,
                    Err(msg) => return Object::Error(msg),
                };
                let val = eval_expression(val_node, Rc::clone(&env));
                if is_error(&val) {
                    return val;
                }
                hash.insert(hash_key, val);
            }
            Object::Hash(hash)
        }
        Expression::WhileExpression { condition, body } => {
            while {
                let cond_val = eval_expression(condition, Rc::clone(&env));
                if is_error(&cond_val) {
                    return cond_val;
                }
                is_truthy(&cond_val)
            } {
                let result = eval_statement(body, Rc::clone(&env));
                if matches!(result, Object::ReturnValue(_)) || matches!(result, Object::Error(_)) {
                    return result;
                }
            }
            Object::Null
        }
        Expression::ImportExpression(path_expr) => {
            let path_val = eval_expression(path_expr, Rc::clone(&env));
            if is_error(&path_val) {
                return path_val;
            }
            if let Object::String(filename) = path_val {
                match std::fs::read_to_string(&filename) {
                    Ok(content) => {
                        let lexer = crate::lexer::Lexer::new(&content);
                        let mut parser = crate::parser::Parser::new(lexer);
                        let program = parser.parse_program();
                        if !parser.errors.is_empty() {
                            return Object::Error(format!("Failed to parse imported file '{}': {:?}", filename, parser.errors));
                        }
                        
                        let module_env = Rc::new(RefCell::new(Environment::new()));
                        crate::builtins::inject_builtins(&module_env);
                        
                        let result = eval_program(&program, Rc::clone(&module_env));
                        if is_error(&result) {
                            return result;
                        }
                        
                        Object::Module(module_env)
                    },
                    Err(e) => Object::Error(format!("Failed to read imported file '{}': {}", filename, e)),
                }
            } else {
                Object::Error(format!("Import path must be a string, got {:?}", path_val))
            }
        }
        Expression::MemberExpression { left, property } => {
            let left_evaluated = eval_expression(left, env);
            if is_error(&left_evaluated) {
                return left_evaluated;
            }
            
            match left_evaluated {
                Object::Module(ref module_env) => {
                    if let Some(val) = module_env.borrow().get(property) {
                        val
                    } else {
                        Object::Error(format!("Module has no property '{}'", property))
                    }
                },
                Object::Instance { class_name: _, ref class_env, ref instance_env } => {
                    // Look in instance env first (data variables)
                    if let Some(val) = instance_env.borrow().get(property) {
                        return val;
                    }
                    // Look in class env next (methods)
                    if let Some(val) = class_env.borrow().get(property) {
                        if matches!(val, Object::Function { .. }) {
                            return Object::BoundMethod {
                                instance: Box::new(left_evaluated.clone()),
                                method: Box::new(val),
                            };
                        }
                        return val;
                    }
                    Object::Error(format!("Instance has no property '{}'", property))
                },
                _ => Object::Error(format!("Cannot access property '{}' on {:?}", property, left_evaluated)),
            }
        }
        Expression::Call { function, arguments } => {
            let function_val = eval_expression(function, Rc::clone(&env));
            if is_error(&function_val) {
                return function_val;
            }
            
            let mut eval_args = vec![];
            for arg in arguments {
                let evaluated = eval_expression(arg, Rc::clone(&env));
                if is_error(&evaluated) {
                    return evaluated;
                }
                eval_args.push(evaluated);
            }
            
            match function_val {
                Object::Function { parameters, body } => {
                    let extended_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env)))); // Functions in Kingcobra capture calling environment (dynamic scope) for simplicity right now
                    
                    for (i, param) in parameters.iter().enumerate() {
                        if i < eval_args.len() {
                            extended_env.borrow_mut().set(param.clone(), eval_args[i].clone());
                        }
                    }

                    let evaluated = eval_statement(&body, extended_env);
                    if let Object::ReturnValue(val) = evaluated {
                        *val
                    } else {
                        evaluated
                    }
                }
                Object::BoundMethod { instance, method } => {
                    if let Object::Function { parameters, body } = *method {
                        let extended_env = Rc::new(RefCell::new(Environment::new_enclosed(Rc::clone(&env))));
                        
                        // Inject `self`
                        extended_env.borrow_mut().set("self".to_string(), *instance);
                        
                        for (i, param) in parameters.iter().enumerate() {
                            if i < eval_args.len() {
                                extended_env.borrow_mut().set(param.clone(), eval_args[i].clone());
                            }
                        }

                        let evaluated = eval_statement(&body, extended_env);
                        if let Object::ReturnValue(val) = evaluated {
                            *val
                        } else {
                            evaluated
                        }
                    } else {
                        Object::Error(format!("Bound method is not a function: {:?}", method))
                    }
                }
                Object::Builtin(func) => func(eval_args),
                Object::Class { name, environment } => {
                    let instance_env = Rc::new(RefCell::new(Environment::new()));
                    Object::Instance {
                        class_name: name,
                        class_env: environment,
                        instance_env,
                    }
                }
                _ => Object::Error(format!("Not a function or class: {:?}", function_val)),
            }
        }
        Expression::Dummy => Object::Error("Parsed dummy expression".to_string()),
    }
}

fn eval_prefix_expression(operator: &Token, right: Object) -> Object {
    match operator {
        Token::Minus => eval_minus_prefix_operator_expression(right),
        Token::New => {
            if matches!(right, Object::Instance { .. }) {
                right
            } else {
                Object::Error(format!("'new' can only be used with class instantiation, got {:?}", right))
            }
        },
        _ => Object::Error(format!("Unknown prefix operator: {:?}", operator)),
    }
}

fn eval_minus_prefix_operator_expression(right: Object) -> Object {
    match right {
        Object::Integer(val) => Object::Integer(-val),
        _ => Object::Error(format!("Unknown operator: -{:?}", right)),
    }
}

fn eval_infix_expression(operator: &Token, left: Object, right: Object) -> Object {
    match (left, right) {
        (Object::Integer(left_val), Object::Integer(right_val)) => {
            eval_integer_infix_expression(operator, left_val, right_val)
        }
        (Object::Boolean(left_val), Object::Boolean(right_val)) => {
            match operator {
                Token::Eq => Object::Boolean(left_val == right_val),
                Token::NotEq => Object::Boolean(left_val != right_val),
                _ => Object::Error(format!("Unknown operator: {:?} for booleans", operator)),
            }
        }
        (Object::String(left_val), Object::String(right_val)) => {
            match operator {
                Token::Plus => Object::String(format!("{}{}", left_val, right_val)),
                Token::Eq => Object::Boolean(left_val == right_val),
                Token::NotEq => Object::Boolean(left_val != right_val),
                _ => Object::Error(format!("Unknown operator: {:?} for strings", operator)),
            }
        }
        (l, r) => Object::Error(format!("Type mismatch: {:?} and {:?}", l, r)),
    }
}

fn eval_integer_infix_expression(operator: &Token, left: i64, right: i64) -> Object {
    match operator {
        Token::Plus => Object::Integer(left + right),
        Token::Minus => Object::Integer(left - right),
        Token::Asterisk => Object::Integer(left * right),
        Token::Slash => {
            if right == 0 {
                return Object::Error("Division by zero".to_string());
            }
            Object::Integer(left / right)
        }
        Token::Lt => Object::Boolean(left < right),
        Token::Gt => Object::Boolean(left > right),
        Token::Eq => Object::Boolean(left == right),
        Token::NotEq => Object::Boolean(left != right),
        _ => Object::Error(format!("Unknown integer operator: {:?}", operator)),
    }
}

fn eval_index_expression(left: Object, index: Object) -> Object {
    match (&left, &index) {
        (Object::Array(elements), Object::Integer(i)) => {
            let max = (elements.len() as i64) - 1;
            if *i < 0 || *i > max {
                Object::Null
            } else {
                elements[*i as usize].clone()
            }
        }
        (Object::Hash(pairs), _) => {
            let hash_key = match index.get_hash_key() {
                Ok(hk) => hk,
                Err(msg) => return Object::Error(msg),
            };
            match pairs.get(&hash_key) {
                Some(val) => val.clone(),
                None => Object::Null,
            }
        }
        _ => Object::Error(format!("Index operator not supported: {:?} [{:?}]", left, index)),
    }
}

fn eval_identifier(name: &str, env: Rc<RefCell<Environment>>) -> Object {
    if let Some(val) = env.borrow().get(name) {
        val
    } else {
        Object::Error(format!("Identifier not found: {}", name))
    }
}

fn is_truthy(obj: &Object) -> bool {
    match obj {
        Object::Null => false,
        Object::Boolean(val) => *val,
        _ => true,
    }
}

fn is_error(obj: &Object) -> bool {
    matches!(obj, Object::Error(_))
}

fn apply_function(function: Object, args: Vec<Object>, env: Rc<RefCell<Environment>>) -> Object {
    match function {
        Object::Function { parameters, body } => {
            let mut extended_env = Environment::new_enclosed(Rc::clone(&env));
            
            for (i, param) in parameters.iter().enumerate() {
                if i < args.len() {
                    extended_env.set(param.clone(), args[i].clone());
                }
            }

            let evaluated = eval_statement(&body, Rc::new(RefCell::new(extended_env)));
            
            if let Object::ReturnValue(val) = evaluated {
                *val
            } else {
                evaluated
            }
        }
        Object::Builtin(builtin_func) => {
            builtin_func(args)
        }
        _ => Object::Error(format!("Not a function: {}", function)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;

    fn test_eval(input: &str) -> Object {
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        let env = Rc::new(RefCell::new(Environment::new()));
        eval_program(&program, env)
    }

    #[test]
    fn test_eval_integer_expression() {
        let tests = vec![
            ("5", Object::Integer(5)),
            ("10", Object::Integer(10)),
            ("-5", Object::Integer(-5)),
            ("-10", Object::Integer(-10)),
            ("5 + 5 + 5 + 5 - 10", Object::Integer(10)),
            ("2 * 2 * 2 * 2 * 2", Object::Integer(32)),
            ("-50 + 100 + -50", Object::Integer(0)),
            ("5 * 2 + 10", Object::Integer(20)),
            ("5 + 2 * 10", Object::Integer(25)),
            ("20 + 2 * -10", Object::Integer(0)),
            ("50 / 2 * 2 + 10", Object::Integer(60)),
            ("2 * (5 + 10)", Object::Integer(30)),
            ("3 * 3 * 3 + 10", Object::Integer(37)),
            ("3 * (3 * 3) + 10", Object::Integer(37)),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", Object::Integer(50)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = vec![
            ("true", Object::Boolean(true)),
            ("false", Object::Boolean(false)),
            ("1 < 2", Object::Boolean(true)),
            ("1 > 2", Object::Boolean(false)),
            ("1 < 1", Object::Boolean(false)),
            ("1 > 1", Object::Boolean(false)),
            ("1 == 1", Object::Boolean(true)),
            ("1 != 1", Object::Boolean(false)),
            ("1 == 2", Object::Boolean(false)),
            ("1 != 2", Object::Boolean(true)),
            ("true == true", Object::Boolean(true)),
            ("false == false", Object::Boolean(true)),
            ("true == false", Object::Boolean(false)),
            ("true != false", Object::Boolean(true)),
            ("false != true", Object::Boolean(true)),
            ("(1 < 2) == true", Object::Boolean(true)),
            ("(1 < 2) == false", Object::Boolean(false)),
            ("(1 > 2) == true", Object::Boolean(false)),
            ("(1 > 2) == false", Object::Boolean(true)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = vec![
            ("when true: return 10 done", Object::Integer(10)),
            ("when false: return 10 done", Object::Null),
            ("when 1: return 10 done", Object::Integer(10)),
            ("when 1 < 2: return 10 done", Object::Integer(10)),
            ("when 1 > 2: return 10 done", Object::Null),
            ("when 1 > 2: return 10 otherwise: return 20 done", Object::Integer(20)),
            ("when 1 < 2: return 10 otherwise: return 20 done", Object::Integer(10)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_return_statements() {
        let tests = vec![
            ("return 10", Object::Integer(10)),
            ("return 10 return 9", Object::Integer(10)),
            ("return 2 * 5", Object::Integer(10)),
            ("9 return 2 * 5 9", Object::Integer(10)),
            (
                "when 10 > 1:
                    when 10 > 1:
                        return 10
                    done
                    return 1
                done",
                Object::Integer(10),
            ),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_error_handling() {
        let tests = vec![
            (
                "5 + true",
                "Type mismatch: Integer(5) and Boolean(true)",
            ),
            (
                "5 + true return 5",
                "Type mismatch: Integer(5) and Boolean(true)",
            ),
            (
                "-true",
                "Unknown operator: -Boolean(true)",
            ),
            (
                "true + false",
                "Unknown operator: Plus for booleans",
            ),
            (
                "foobar",
                "Identifier not found: foobar",
            ),
        ];

        for (input, expected) in tests {
            let result = test_eval(input);
            if let Object::Error(msg) = result {
                assert_eq!(msg, expected);
            } else {
                panic!("Expected Error, got {:?}", result);
            }
        }
    }

    #[test]
    fn test_let_statements() {
        let tests = vec![
            ("let a = 5 a", Object::Integer(5)),
            ("let a = 5 * 5 a", Object::Integer(25)),
            ("let a = 5 let b = a b", Object::Integer(5)),
            ("let a = 5 let b = a let c = a + b + 5 c", Object::Integer(15)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_function_application() {
        let tests = vec![
            ("let identity = fn(x): return x done identity(5)", Object::Integer(5)),
            ("let identity = fn(x): return x done identity(identity(5))", Object::Integer(5)),
            ("let double = fn(x): return x * 2 done double(5)", Object::Integer(10)),
            ("let add = fn(x, y): return x + y done add(5, 5)", Object::Integer(10)),
            ("let add = fn(x, y): return x + y done add(5 + 5, add(5, 5))", Object::Integer(20)),
            ("fn(x): return x done(5)", Object::Integer(5)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_array_literals() {
        let input = "[1, 2 * 2, 3 + 3]";
        let evaluated = test_eval(input);
        if let Object::Array(elements) = evaluated {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], Object::Integer(1));
            assert_eq!(elements[1], Object::Integer(4));
            assert_eq!(elements[2], Object::Integer(6));
        } else {
            panic!("Expected Array, got {:?}", evaluated);
        }
    }

    #[test]
    fn test_array_index_expressions() {
        let tests = vec![
            ("[1, 2, 3][0]", Object::Integer(1)),
            ("[1, 2, 3][1]", Object::Integer(2)),
            ("[1, 2, 3][2]", Object::Integer(3)),
            ("let my_array = [1, 2, 3] let i = 0 my_array[i]", Object::Integer(1)),
            ("[1, 2, 3][1 + 1]", Object::Integer(3)),
            ("let my_array = [1, 2, 3] my_array[2]", Object::Integer(3)),
            ("let my_array = [1, 2, 3] my_array[0] + my_array[1] + my_array[2]", Object::Integer(6)),
            ("let my_array = [1, 2, 3] let i = my_array[0] my_array[i]", Object::Integer(2)),
            ("[1, 2, 3][3]", Object::Null),
            ("[1, 2, 3][-1]", Object::Null),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_hash_literals() {
        let input = r#"
        let two = "two"
        {
            "one": 10 - 9,
            two: 1 + 1,
            "thr" + "ee": 6 / 2,
            4: 4,
            true: 5,
            false: 6
        }
        "#;

        let evaluated = test_eval(input);
        if let Object::Hash(pairs) = evaluated {
            assert_eq!(pairs.len(), 6);
            
            assert_eq!(pairs.get(&HashKey::String("one".to_string())).unwrap(), &Object::Integer(1));
            assert_eq!(pairs.get(&HashKey::String("two".to_string())).unwrap(), &Object::Integer(2));
            assert_eq!(pairs.get(&HashKey::String("three".to_string())).unwrap(), &Object::Integer(3));
            assert_eq!(pairs.get(&HashKey::Integer(4)).unwrap(), &Object::Integer(4));
            assert_eq!(pairs.get(&HashKey::Boolean(true)).unwrap(), &Object::Integer(5));
            assert_eq!(pairs.get(&HashKey::Boolean(false)).unwrap(), &Object::Integer(6));
        } else {
            panic!("Expected Hash, got {:?}", evaluated);
        }
    }

    #[test]
    fn test_hash_index_expressions() {
        let tests = vec![
            ("{\"foo\": 5}[\"foo\"]", Object::Integer(5)),
            ("{\"foo\": 5}[\"bar\"]", Object::Null),
            ("let key = \"foo\" {\"foo\": 5}[key]", Object::Integer(5)),
            ("{}[\"foo\"]", Object::Null),
            ("{5: 5}[5]", Object::Integer(5)),
            ("{true: 5}[true]", Object::Integer(5)),
            ("{false: 5}[false]", Object::Integer(5)),
        ];

        for (input, expected) in tests {
            assert_eq!(test_eval(input), expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_while_loops() {
        let input = r#"
        let i = 0
        let sum = 0
        while i < 5:
            let i = i + 1
            let sum = sum + i
        done
        sum
        "#;
        
        assert_eq!(test_eval(input), Object::Integer(15));
    }
}
