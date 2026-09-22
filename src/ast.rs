use crate::token::Token;

/// A Program is just a list of Statements that make up the entire file
#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A Statement is an instruction that DOES NOT return a value (like defining a variable)
#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    /// E.g., `let x = 5` or `let self.health = 10`
    Let { name: Expression, value: Expression },
    /// E.g., `return x`
    Return(Expression),
    /// E.g., `5 + 5` on a line by itself
    Expression(Expression),
    /// E.g., The code inside a `when` block or function
    Block(Vec<Statement>),
    /// E.g., `class Player: ... done`
    ClassStatement {
        name: String,
        body: Box<Statement>,
    },
}

/// An Expression is a piece of code that DOES return a value (like a number or a math problem)
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(String),
    Integer(i64),
    String(String),
    Boolean(bool),
    
    /// E.g., `-5` or `!true`
    Prefix { 
        operator: Token, 
        right: Box<Expression> 
    },
    
    /// E.g., `5 + 5` or `x < 10`
    Infix { 
        left: Box<Expression>, 
        operator: Token, 
        right: Box<Expression> 
    },
    
    /// E.g., `when x < 10: ... otherwise: ... done`
    If { 
        condition: Box<Expression>, 
        consequence: Box<Statement>, 
        alternative: Option<Box<Statement>> 
    },
    
    /// E.g., `fn(x, y): ... done`
    Function { 
        parameters: Vec<String>, 
        body: Box<Statement> 
    },
    
    /// E.g., `add(2, 3)`
    Call { 
        function: Box<Expression>, 
        arguments: Vec<Expression> 
    },
    
    /// E.g., `[1, 2, 3]`
    ArrayLiteral(Vec<Expression>),

    /// E.g., `my_list[0]`
    IndexExpression {
        left: Box<Expression>,
        index: Box<Expression>
    },

    /// E.g., `{"name": "shivam"}`
    HashLiteral(Vec<(Expression, Expression)>),

    /// E.g., `while x < 10: ... done`
    WhileExpression {
        condition: Box<Expression>,
        body: Box<Statement>,
    },

    /// E.g., `import "math.kc"`
    ImportExpression(Box<Expression>),

    /// E.g., `math.add`
    MemberExpression {
        left: Box<Expression>,
        property: String,
    },

    /// Used when the parser encounters a syntax error but wants to keep checking the rest of the file
    Dummy,
}
