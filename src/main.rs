use kingcobra::repl;
use std::env;
use std::fs;
use kingcobra::lexer::Lexer;
use kingcobra::parser::Parser;
use kingcobra::evaluator::eval_program;
use kingcobra::environment::Environment;
use std::rc::Rc;
use std::cell::RefCell;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Run a file
        let filename = &args[1];
        let contents = match fs::read_to_string(filename) {
            Ok(c) => c,
            Err(e) => {
                println!("Error reading file {}: {}", filename, e);
                return;
            }
        };

        let lexer = Lexer::new(&contents);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();

        if !parser.errors.is_empty() {
            println!("Syntax Errors:");
            for err in parser.errors {
                println!("\t{}", err);
            }
            return;
        }

        let env = Rc::new(RefCell::new(Environment::new()));
        kingcobra::builtins::inject_builtins(&env);
        
        let evaluated = eval_program(&program, Rc::clone(&env));
        
        if let kingcobra::object::Object::Error(err) = evaluated {
            println!("Runtime Error: {}", err);
        }
    } else {
        // Start REPL
        repl::start();
    }
}
