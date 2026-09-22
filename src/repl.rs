use std::io::{self, Write};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::evaluator::eval_program;
use crate::environment::Environment;
use std::rc::Rc;
use std::cell::RefCell;

const PROMPT: &str = "🐍>> ";
const KINGCOBRA_ASCII: &str = r#"
    __
   {0O}
   \__/
   /^/
  ( (
   \_\_____
   (_______)
   (_________()Oo
"#;

pub fn start() {
    println!("{}", KINGCOBRA_ASCII);
    println!("Welcome to the Kingcobra Programming Language!");
    println!("Type your code below. Type 'exit' to quit.");
    
    let env = Rc::new(RefCell::new(Environment::new()));
    
    // We import builtins into the global environment here
    crate::builtins::inject_builtins(&env);

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        print!("{}", PROMPT);
        stdout.flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            println!("Error reading input");
            return;
        }

        let input = input.trim();
        if input == "exit" {
            println!("Goodbye!");
            break;
        }

        if input.is_empty() {
            continue;
        }

        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let program = parser.parse_program();

        if !parser.errors.is_empty() {
            println!("Whoops! We ran into some snake bites (parsing errors):");
            for err in parser.errors {
                println!("\t{}", err);
            }
            continue;
        }

        let evaluated = eval_program(&program, Rc::clone(&env));
        
        // Don't print "null" for let statements or block evaluations that return nothing
        if !matches!(evaluated, crate::object::Object::Null) {
            println!("{}", evaluated);
        }
    }
}
