use compiler::ast::traits::Walkable;
use compiler::errors::CompilerError;

pub struct ValueNode {
    pub value: u32,
}

impl ValueNode {
    pub(crate) fn new(value: u32) -> Self {
        Self { value, }
    }
}

impl Walkable for ValueNode {
    fn walk(&mut self, visitor: Box<dyn Fn(&dyn Walkable) -> Result<(), CompilerError<'_>>>) -> Result<(), CompilerError<'_>> {
        visitor(self)
    }
}

