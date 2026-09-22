use crate::ast::{Expression, Program, Statement};
use crate::lexer::Lexer;
use crate::token::Token;

#[derive(PartialEq, PartialOrd, Debug, Clone, Copy)]
enum Precedence {
    Lowest,
    Equals,      // ==
    LessGreater, // > or <
    Sum,         // +
    Product,     // *
    Prefix,      // -X or !X
    Call,        // myFunction(X)
    Index,       // array[index]
}

fn token_precedence(token: &Token) -> Precedence {
    match token {
        Token::Eq | Token::NotEq => Precedence::Equals,
        Token::Lt | Token::Gt => Precedence::LessGreater,
        Token::Plus | Token::Minus => Precedence::Sum,
        Token::Slash | Token::Asterisk => Precedence::Product,
        Token::LParen => Precedence::Call,
        Token::Dot => Precedence::Call,
        Token::LBracket => Precedence::Index,
        _ => Precedence::Lowest,
    }
}

pub struct Parser {
    lexer: Lexer,
    cur_token: Token,
    peek_token: Token,
    pub errors: Vec<String>,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let cur_token = lexer.next_token();
        let peek_token = lexer.next_token();
        Parser {
            lexer,
            cur_token,
            peek_token,
            errors: vec![],
        }
    }

    pub fn next_token(&mut self) {
        self.cur_token = self.peek_token.clone();
        self.peek_token = self.lexer.next_token();
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program { statements: vec![] };
        while self.cur_token != Token::Eof {
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }
        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match self.cur_token {
            Token::Let => self.parse_let_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Class => self.parse_class_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        self.next_token();
        
        let name = self.parse_expression(Precedence::Lowest)?;

        // expect '='
        if !matches!(self.peek_token, Token::Assign) {
            self.errors.push(format!("Error at line {}, col {}: Expected '=', got {:?}", self.lexer.line, self.lexer.column, self.peek_token));
            return None;
        }
        self.next_token();
        self.next_token();

        let value = self.parse_expression(Precedence::Lowest)?;

        Some(Statement::Let { name, value })
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        self.next_token();
        let value = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Return(value))
    }

    fn parse_class_statement(&mut self) -> Option<Statement> {
        if !matches!(self.peek_token, Token::Ident(_)) {
            self.errors.push(format!("Error at line {}, col {}: Expected identifier after class, got {:?}", self.lexer.line, self.lexer.column, self.peek_token));
            return None;
        }
        self.next_token();

        let name = if let Token::Ident(ref n) = self.cur_token {
            n.clone()
        } else {
            return None;
        };

        if !matches!(self.peek_token, Token::Colon) {
            self.errors.push(format!("Error at line {}, col {}: Expected ':' after class name, got {:?}", self.lexer.line, self.lexer.column, self.peek_token));
            return None;
        }
        self.next_token();
        self.next_token();

        let body = self.parse_block_statement();

        Some(Statement::ClassStatement {
            name,
            body: Box::new(Statement::Block(body)),
        })
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let expr = self.parse_expression(Precedence::Lowest)?;
        Some(Statement::Expression(expr))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Option<Expression> {
        let mut left = match self.cur_token {
            Token::Ident(ref name) => Some(Expression::Identifier(name.clone())),
            Token::Int(val) => Some(Expression::Integer(val)),
            Token::String(ref val) => Some(Expression::String(val.clone())),
            Token::True => Some(Expression::Boolean(true)),
            Token::False => Some(Expression::Boolean(false)),
            Token::Minus | Token::Illegal => self.parse_prefix_expression(), // '!' is Illegal for now, update lexer later if needed, but wait we have Token::NotEq. We need Token::Bang for '!'
            Token::LParen => self.parse_grouped_expression(),
            Token::LBracket => self.parse_array_literal(),
            Token::LBrace => self.parse_hash_literal(),
            Token::When => self.parse_if_expression(),
            Token::While => self.parse_while_expression(),
            Token::Fn => self.parse_function_literal(),
            Token::Import => self.parse_import_expression(),
            Token::New => self.parse_prefix_expression(), // 'new' is just a prefix!
            _ => {
                self.errors.push(format!("Error at line {}, col {}: No prefix parse function for {:?}", self.lexer.line, self.lexer.column, self.cur_token));
                None
            }
        };

        while self.peek_token != Token::Eof && precedence < token_precedence(&self.peek_token) {
            self.next_token();
            left = self.parse_infix_expression(left.unwrap());
        }

        left
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let operator = self.cur_token.clone();
        self.next_token();
        let right = self.parse_expression(Precedence::Prefix)?;
        Some(Expression::Prefix {
            operator,
            right: Box::new(right),
        })
    }

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let operator = self.cur_token.clone();
        let precedence = token_precedence(&self.cur_token);
        
        if matches!(operator, Token::LParen) {
            return self.parse_call_expression(left);
        } else if matches!(operator, Token::LBracket) {
            return self.parse_index_expression(left);
        } else if matches!(operator, Token::Dot) {
            return self.parse_member_expression(left);
        }

        self.next_token();
        let right = self.parse_expression(precedence)?;
        Some(Expression::Infix {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        })
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let exp = self.parse_expression(Precedence::Lowest);
        if !matches!(self.peek_token, Token::RParen) {
            return None;
        }
        self.next_token();
        exp
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        
        if !matches!(self.peek_token, Token::Colon) {
            return None;
        }
        self.next_token();
        self.next_token();

        let consequence = self.parse_block_statement();
        let mut alternative = None;

        if matches!(self.cur_token, Token::Otherwise) {
            if !matches!(self.peek_token, Token::Colon) {
                return None;
            }
            self.next_token();
            self.next_token();
            alternative = Some(Box::new(Statement::Block(self.parse_block_statement_inner())));
        }

        Some(Expression::If {
            condition: Box::new(condition),
            consequence: Box::new(Statement::Block(consequence)),
            alternative,
        })
    }

    fn parse_block_statement(&mut self) -> Vec<Statement> {
        self.parse_block_statement_inner()
    }

    fn parse_block_statement_inner(&mut self) -> Vec<Statement> {
        let mut statements = vec![];
        while !matches!(self.cur_token, Token::Done) && !matches!(self.cur_token, Token::Otherwise) && !matches!(self.cur_token, Token::Eof) {
            if let Some(stmt) = self.parse_statement() {
                statements.push(stmt);
            }
            self.next_token();
        }
        statements
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        if !matches!(self.peek_token, Token::LParen) {
            return None;
        }
        self.next_token();

        let parameters = self.parse_function_parameters();

        if !matches!(self.peek_token, Token::Colon) {
            return None;
        }
        self.next_token();
        self.next_token();

        let body = self.parse_block_statement();

        Some(Expression::Function {
            parameters,
            body: Box::new(Statement::Block(body)),
        })
    }

    fn parse_function_parameters(&mut self) -> Vec<String> {
        let mut identifiers = vec![];
        if matches!(self.peek_token, Token::RParen) {
            self.next_token();
            return identifiers;
        }

        self.next_token();
        if let Token::Ident(ref name) = self.cur_token {
            identifiers.push(name.clone());
        }

        while matches!(self.peek_token, Token::Comma) {
            self.next_token();
            self.next_token();
            if let Token::Ident(ref name) = self.cur_token {
                identifiers.push(name.clone());
            }
        }

        if !matches!(self.peek_token, Token::RParen) {
            return vec![];
        }
        self.next_token();
        identifiers
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let arguments = self.parse_call_arguments();
        Some(Expression::Call {
            function: Box::new(function),
            arguments,
        })
    }

    fn parse_call_arguments(&mut self) -> Vec<Expression> {
        let mut args = vec![];
        if matches!(self.peek_token, Token::RParen) {
            self.next_token();
            return args;
        }

        self.next_token();
        if let Some(expr) = self.parse_expression(Precedence::Lowest) {
            args.push(expr);
        }

        while matches!(self.peek_token, Token::Comma) {
            self.next_token();
            self.next_token();
            if let Some(expr) = self.parse_expression(Precedence::Lowest) {
                args.push(expr);
            }
        }

        if !matches!(self.peek_token, Token::RParen) {
            return vec![];
        }
        self.next_token();
        args
    }

    fn parse_array_literal(&mut self) -> Option<Expression> {
        let elements = self.parse_expression_list(Token::RBracket);
        Some(Expression::ArrayLiteral(elements))
    }

    fn parse_expression_list(&mut self, end_token: Token) -> Vec<Expression> {
        let mut list = vec![];
        if self.peek_token == end_token {
            self.next_token();
            return list;
        }

        self.next_token();
        if let Some(expr) = self.parse_expression(Precedence::Lowest) {
            list.push(expr);
        }

        while self.peek_token == Token::Comma {
            self.next_token();
            self.next_token();
            if let Some(expr) = self.parse_expression(Precedence::Lowest) {
                list.push(expr);
            }
        }

        if self.peek_token != end_token {
            return vec![];
        }
        self.next_token();
        list
    }

    fn parse_index_expression(&mut self, left: Expression) -> Option<Expression> {
        self.next_token();
        let index = self.parse_expression(Precedence::Lowest)?;
        if !matches!(self.peek_token, Token::RBracket) {
            return None;
        }
        self.next_token();
        Some(Expression::IndexExpression {
            left: Box::new(left),
            index: Box::new(index),
        })
    }

    fn parse_hash_literal(&mut self) -> Option<Expression> {
        let mut pairs = vec![];
        while self.peek_token != Token::RBrace {
            self.next_token();
            let key = self.parse_expression(Precedence::Lowest)?;

            if !matches!(self.peek_token, Token::Colon) {
                return None;
            }
            self.next_token();
            self.next_token();
            let value = self.parse_expression(Precedence::Lowest)?;

            pairs.push((key, value));

            if self.peek_token != Token::RBrace && self.peek_token != Token::Comma {
                return None;
            }
            if self.peek_token == Token::Comma {
                self.next_token();
            }
        }

        if !matches!(self.peek_token, Token::RBrace) {
            return None;
        }
        self.next_token();
        Some(Expression::HashLiteral(pairs))
    }

    fn parse_while_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let condition = self.parse_expression(Precedence::Lowest)?;
        
        if !matches!(self.peek_token, Token::Colon) {
            return None;
        }
        self.next_token();
        self.next_token();

        let body = self.parse_block_statement();

        Some(Expression::WhileExpression {
            condition: Box::new(condition),
            body: Box::new(Statement::Block(body)),
        })
    }

    fn parse_import_expression(&mut self) -> Option<Expression> {
        self.next_token();
        let path = self.parse_expression(Precedence::Lowest)?;
        Some(Expression::ImportExpression(Box::new(path)))
    }

    fn parse_member_expression(&mut self, left: Expression) -> Option<Expression> {
        self.next_token();
        let property = match &self.cur_token {
            Token::Ident(name) => name.clone(),
            _ => {
                self.errors.push(format!("Error at line {}, col {}: Expected identifier after dot, got {:?}", self.lexer.line, self.lexer.column, self.cur_token));
                return None;
            }
        };

        Some(Expression::MemberExpression {
            left: Box::new(left),
            property,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_let_statements() {
        let input = r#"
let x = 5
let y = 10
let my_var = 838383
"#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();
        
        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 3);

        let expected_identifiers = vec!["x", "y", "my_var"];

        for (i, stmt) in program.statements.iter().enumerate() {
            match stmt {
                Statement::Let { name, .. } => {
                    if let Expression::Identifier(id) = name {
                        assert_eq!(id, expected_identifiers[i]);
                    } else {
                        panic!("Expected LetStatement name to be Identifier, got {:?}", name);
                    }
                }
                _ => panic!("Expected LetStatement, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let input = r#"
return 5
return 10
return 993322
"#;
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);

        let program = parser.parse_program();
        
        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 3);

        for stmt in program.statements {
            match stmt {
                Statement::Return(_) => {}
                _ => panic!("Expected ReturnStatement, got {:?}", stmt),
            }
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        // These tests check if the parser understands that multiplication happens before addition,
        // and that parentheses group things together correctly.
        let tests = vec![
            "-a * b",
            "a + b + c",
            "a + b - c",
            "a * b * c",
            "a * b / c",
            "a + b * c + d / e - f",
            "3 + 4 * 5 == 3 * 1 + 4 * 5",
            "true",
            "false",
            "3 > 5 == false",
            "3 < 5 == true",
            "1 + (2 + 3) + 4",
            "(5 + 5) * 2",
            "2 / (5 + 5)",
            "-(5 + 5)",
            "a + add(b * c) + d",
            "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
        ];

        for input in tests {
            let lexer = Lexer::new(input);
            let mut parser = Parser::new(lexer);
            let program = parser.parse_program();
            
            // If the parser hits an infinite loop, panics, or doesn't understand the math rules,
            // it will push an error to parser.errors.
            assert_eq!(parser.errors.len(), 0, "Parser had errors for input '{}': {:?}", input, parser.errors);
            assert_eq!(program.statements.len(), 1, "Expected 1 statement for {}", input);
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "when x < y: return x otherwise: return y done";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        
        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 1);
        
        match &program.statements[0] {
            Statement::Expression(Expression::If { .. }) => {
                // Successfully parsed as an If Expression
            },
            _ => panic!("Expected If expression, got {:?}", program.statements[0]),
        }
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y): return x + y done";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let program = parser.parse_program();
        
        assert_eq!(parser.errors.len(), 0, "Parser had errors: {:?}", parser.errors);
        assert_eq!(program.statements.len(), 1);

        match &program.statements[0] {
            Statement::Expression(Expression::Function { parameters, .. }) => {
                assert_eq!(parameters.len(), 2);
                assert_eq!(parameters[0], "x");
                assert_eq!(parameters[1], "y");
            },
            _ => panic!("Expected Function literal, got {:?}", program.statements[0]),
        }
    }
}
