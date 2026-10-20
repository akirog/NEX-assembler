use compiler::errors::CompilerError;


/// Trait for all ast nodes.
/// Allows node to be traversed by different stages.
pub trait Walkable {
    /// Calls walk on all walkable members with `visitor`, then calls visitor on self.
    /// Basically call this on a node, and it will run `visitor` on self and all its members.
    fn walk(&mut self, visitor: Box<dyn Fn(&dyn Walkable) -> Result<(), CompilerError<'_>>>) -> Result<(), CompilerError<'_>>;
}