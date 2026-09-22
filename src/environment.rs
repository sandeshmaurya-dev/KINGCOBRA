use std::collections::HashMap;
use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;

/// The Environment is essentially the computer's RAM/Scope during execution.
/// It keeps track of all variables that have been assigned.
#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    
    // The "outer" environment is used for scope. 
    // If you are inside a function, and you ask for variable 'x',
    // it will first check the function's local store. If it doesn't find it,
    // it will look at the outer store (the global variables).
    pub outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(obj) => Some(obj.clone()),
            None => {
                // If it's not in the local scope, check the outer scope!
                if let Some(outer_env) = &self.outer {
                    outer_env.borrow().get(name)
                } else {
                    None
                }
            }
        }
    }

    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val.clone());
        val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_environment_scoping() {
        // Create global scope
        let global = Rc::new(RefCell::new(Environment::new()));
        global.borrow_mut().set("x".to_string(), Object::Integer(10));
        global.borrow_mut().set("y".to_string(), Object::Integer(20));

        // Create a local scope (like inside a function)
        let local = Environment::new_enclosed(Rc::clone(&global));

        // The local scope should have access to global variables
        assert_eq!(local.get("x"), Some(Object::Integer(10)));
        assert_eq!(local.get("y"), Some(Object::Integer(20)));
        assert_eq!(local.get("z"), None); // z doesn't exist
    }

    #[test]
    fn test_environment_shadowing() {
        let global = Rc::new(RefCell::new(Environment::new()));
        global.borrow_mut().set("x".to_string(), Object::Integer(10));

        let mut local = Environment::new_enclosed(Rc::clone(&global));
        // The local scope defines its own 'x' (shadowing the global one)
        local.set("x".to_string(), Object::Integer(99));

        // The local scope gets its own 'x'
        assert_eq!(local.get("x"), Some(Object::Integer(99)));
        
        // But the global scope's 'x' is unaffected
        assert_eq!(global.borrow().get("x"), Some(Object::Integer(10)));
    }
}
