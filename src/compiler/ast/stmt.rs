use std::fmt::{write, Arguments, Debug, Display, Formatter};
use compiler::ast::block::BlockNode;
use compiler::ast::expr::ExprNode;
use compiler::ast::{Field, ValueType};
use compiler::errors::Span;

pub struct StmtNode<'a> {
    pub kind: StmtKind<'a>,
    pub span: Span<'a>,
}

impl<'a> StmtNode<'a> {
    pub fn new(span: Span<'a>, kind: StmtKind<'a>) -> StmtNode<'a> {
        StmtNode { kind, span }
    }
}

impl<'a> Debug for StmtNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl<'a> Display for StmtNode<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            StmtKind::FunctionDeclaration { name, ret_type, arguments, body } => {
                write!(f, "fn {}({:?}) -> {} {}", name, arguments, ret_type, body)
            },
            StmtKind::StructDefinition { name, fields } => {
                write!(f, "struct {} {{", name)?;
                for field in fields {
                    write!(f, " {}", field)?;
                }
                write!(f, "}}")
            }
            StmtKind::Assignment { target, value, .. } => {
                write!(f, "{} = {}", target, value)
            },
            StmtKind::Declaration { var_type, name, value } => {
                write!(f, "let {}: {} = {:?}", name, var_type, value)
            },
            StmtKind::IfStatement { condition, body } => {
                write!(f, "if {} {}", condition, body)
            },
            StmtKind::WhileLoop { condition, body } => {
                write!(f, "while {} {}", condition, body)
            },
            StmtKind::ForLoop { init, condition, update, body } => {
                write!(f, "for {} {} {} {}", init, condition, update, body)
            },
            StmtKind::Loop { body } => {
                write!(f, "loop {}", body)
            }
            StmtKind::Expression( expression ) => {
                write!(f, "expr {}", expression)
            },
        }
    }
}


#[derive(Debug)]
pub enum StmtKind<'a> {
    /// A function declaration
    FunctionDeclaration {
        name: String,
        ret_type: ValueType,
        arguments: Vec<Field>,
        body: BlockNode<'a>
    },

    /// Struct definition
    StructDefinition {
        name: String,
        fields: Vec<Field>,
    },

    /// An if statement
    IfStatement {
        condition: ExprNode<'a>,
        body: BlockNode<'a>
    },

    /// A while loop
    WhileLoop {
        condition: ExprNode<'a>,
        body: BlockNode<'a>
    },

    /// For loop
    ForLoop {
        init: Box<StmtNode<'a>>,
        condition: ExprNode<'a>,
        update: Box<StmtNode<'a>>,
        body: BlockNode<'a>
    },

    /// Unconditional loop
    Loop {
        body: BlockNode<'a>
    },

    /// An assignment of an expression to some expression
    Assignment {
        target: Box<ExprNode<'a>>,
        value: Box<ExprNode<'a>>
    },

    /// A variable declaration
    Declaration {
        name: String,
        value: Option<Box<ExprNode<'a>>>,
        var_type: ValueType,
    },

    /// Expression statement, for stuff like function calls outside of statements
    Expression(Box<ExprNode<'a>>),
}