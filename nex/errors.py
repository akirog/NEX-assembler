from .compiler.frame_classes import Span, Position

class NexError(Exception):
    """Base class for exceptions used by nex."""

class CompileError(NexError):
    def __init__(self, message, span: Span = None, hint: str = None):
        self.message, self.span, self.hint = message, span, hint
        super().__init__(message)


class AssembleError(NexError):
    def __init__(self, message, file=None, line=None):
        self.message, self.file, self.line = message, file, line
        super().__init__(message)

    def __str__(self):
        loc = f"{self.file}:{self.line}: " if self.file and self.line else ""
        return f"{loc}{self.message}"