use crate::environment::Environment;
use crate::object::Object;
use std::cell::RefCell;
use std::rc::Rc;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn inject_builtins(env: &Rc<RefCell<Environment>>) {
    // 'print' function: takes any number of arguments, prints them to stdout, returns Null
    env.borrow_mut().set("print".to_string(), Object::Builtin(|args: Vec<Object>| -> Object {
        for arg in args {
            print!("{} ", arg);
        }
        println!();
        Object::Null
    }));

    // 'len' function: takes exactly 1 string, returns its length as an integer
    env.borrow_mut().set("len".to_string(), Object::Builtin(|args: Vec<Object>| -> Object {
        if args.len() != 1 {
            return Object::Error(format!("Wrong number of arguments for len. Got {}, expected 1", args.len()));
        }

        match &args[0] {
            Object::String(s) => Object::Integer(s.len() as i64),
            _ => Object::Error(format!("Argument to 'len' not supported: {}", args[0])),
        }
    }));

    // 'read_file(path)'
    env.borrow_mut().set("read_file".to_string(), Object::Builtin(|args: Vec<Object>| -> Object {
        if args.len() != 1 {
            return Object::Error(format!("Wrong number of arguments for read_file. Got {}, expected 1", args.len()));
        }
        if let Object::String(path) = &args[0] {
            match fs::read_to_string(path) {
                Ok(content) => Object::String(content),
                Err(e) => Object::Error(format!("Failed to read file {}: {}", path, e)),
            }
        } else {
            Object::Error("Argument to read_file must be a string".to_string())
        }
    }));

    // 'write_file(path, content)'
    env.borrow_mut().set("write_file".to_string(), Object::Builtin(|args: Vec<Object>| -> Object {
        if args.len() != 2 {
            return Object::Error(format!("Wrong number of arguments for write_file. Got {}, expected 2", args.len()));
        }
        if let (Object::String(path), Object::String(content)) = (&args[0], &args[1]) {
            match fs::write(path, content) {
                Ok(_) => Object::Boolean(true),
                Err(e) => Object::Error(format!("Failed to write file {}: {}", path, e)),
            }
        } else {
            Object::Error("Arguments to write_file must be strings".to_string())
        }
    }));

    // 'time()' returns unix timestamp in seconds
    env.borrow_mut().set("time".to_string(), Object::Builtin(|_| -> Object {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => Object::Integer(n.as_secs() as i64),
            Err(_) => Object::Error("SystemTime before UNIX EPOCH!".to_string()),
        }
    }));

    // 'random()' returns a pseudo-random integer based on system time
    env.borrow_mut().set("random".to_string(), Object::Builtin(|_| -> Object {
        match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(n) => {
                let nanos = n.as_nanos() as i64;
                let pseudo_random = (nanos ^ (nanos << 13) ^ (nanos >> 7)) % 1000000;
                Object::Integer(pseudo_random.abs())
            },
            Err(_) => Object::Integer(42),
        }
    }));
}
