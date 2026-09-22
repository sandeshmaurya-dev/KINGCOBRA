use crate::ast::Statement;
use crate::environment::Environment;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;
use std::collections::HashMap;

/// The Object enum represents every piece of data inside Kingcobra while it is running.
/// If you type `let x = 10`, Kingcobra stores `Object::Integer(10)` in RAM.
#[derive(Debug, PartialEq, Clone)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Null, // Used when a function returns nothing, or a variable is empty
    ReturnValue(Box<Object>), // A wrapper to tell the evaluator to stop running code and return
    Error(String), // Used to stop the program if the user does impossible math (like "hello" + 5)
    
    // Functions are "First-Class" in Kingcobra, meaning they are stored in memory just like numbers
    Function {
        parameters: Vec<String>,
        body: Box<Statement>,
        // Note: We will add the 'Environment' (Memory Scope) to this in the next step!
    },
    
    // Builtin functions are native Rust functions exposed to Kingcobra (like `print` or `len`)
    Builtin(fn(Vec<Object>) -> Object),

    Array(Vec<Object>),
    Hash(HashMap<HashKey, Object>),
    Module(Rc<RefCell<Environment>>),
    Class {
        name: String,
        environment: Rc<RefCell<Environment>>,
    },
    Instance {
        class_name: String,
        class_env: Rc<RefCell<Environment>>,
        instance_env: Rc<RefCell<Environment>>,
    },
    BoundMethod {
        instance: Box<Object>,
        method: Box<Object>,
    },
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum HashKey {
    Integer(i64),
    String(String),
    Boolean(bool),
}

impl Object {
    pub fn get_hash_key(&self) -> Result<HashKey, String> {
        match self {
            Object::Integer(i) => Ok(HashKey::Integer(*i)),
            Object::String(s) => Ok(HashKey::String(s.clone())),
            Object::Boolean(b) => Ok(HashKey::Boolean(*b)),
            _ => Err(format!("Unusable as hash key: {:?}", self)),
        }
    }
}

/// This allows us to easily print Kingcobra objects to the terminal in a readable way
impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Object::Integer(i) => write!(f, "{}", i),
            Object::String(s) => write!(f, "{}", s),
            Object::Boolean(b) => write!(f, "{}", b),
            Object::Null => write!(f, "null"),
            Object::ReturnValue(val) => write!(f, "{}", val),
            Object::Error(err) => write!(f, "ERROR: {}", err),
            Object::Function { parameters, .. } => {
                write!(f, "fn({}) {{ ... }}", parameters.join(", "))
            }
            Object::Builtin(_) => write!(f, "native built-in function"),
            Object::Array(elements) => {
                let formatted_elements: Vec<String> = elements.iter().map(|e| format!("{}", e)).collect();
                write!(f, "[{}]", formatted_elements.join(", "))
            }
            Object::Hash(pairs) => {
                let mut formatted_pairs = vec![];
                for (key, val) in pairs {
                    let key_str = match key {
                        HashKey::Integer(i) => i.to_string(),
                        HashKey::String(s) => format!("\"{}\"", s),
                        HashKey::Boolean(b) => b.to_string(),
                    };
                    formatted_pairs.push(format!("{}: {}", key_str, val));
                }
                write!(f, "{{{}}}", formatted_pairs.join(", "))
            }
            Object::Module(_) => write!(f, "<module>"),
            Object::Class { name, .. } => write!(f, "<class {}>", name),
            Object::Instance { class_name, .. } => write!(f, "<instance {}>", class_name),
            Object::BoundMethod { method, .. } => write!(f, "<bound method: {}>", method),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Statement;

    #[test]
    fn test_object_formatting() {
        let tests = vec![
            (Object::Integer(5), "5"),
            (Object::Integer(-10), "-10"),
            (Object::Boolean(true), "true"),
            (Object::Boolean(false), "false"),
            (Object::String("hello world".to_string()), "hello world"),
            (Object::Null, "null"),
            (Object::ReturnValue(Box::new(Object::Integer(99))), "99"),
            (Object::Error("Type mismatch".to_string()), "ERROR: Type mismatch"),
        ];

        for (obj, expected) in tests {
            assert_eq!(format!("{}", obj), expected, "Formatting failed for {:?}", obj);
        }
    }

    #[test]
    fn test_function_formatting() {
        let func = Object::Function {
            parameters: vec!["x".to_string(), "y".to_string()],
            body: Box::new(Statement::Block(vec![])),
        };
        assert_eq!(format!("{}", func), "fn(x, y) { ... }");
    }
}
