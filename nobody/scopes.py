"""Lexical environment and closure cells for experimental Nobody AST runtime."""
class ExecutionError(RuntimeError):
    pass

class Scope:
    def __init__(self,parent=None):
        self.parent=parent
        self.values={}
        self.types={}
    def owner(self,name):
        if name in self.values:return self
        return self.parent.owner(name) if self.parent is not None else None
    def get(self,name):
        frame=self.owner(name)
        if frame is None:raise ExecutionError('Unknown variable or function '+name)
        return frame.values[name]
    def declare(self,name,value,expected=None):
        if name in self.values:raise ExecutionError('Duplicate declaration '+name)
        if expected is not None and type(value) is not expected:
            raise ExecutionError('Type mismatch for '+name)
        self.values[name]=value
        if expected is not None:self.types[name]=expected
    def assign(self,name,value):
        frame=self.owner(name)
        if frame is None:raise ExecutionError('Assignment to undeclared variable '+name)
        expected=frame.types.get(name)
        if expected is not None and type(value) is not expected:
            raise ExecutionError('Type mismatch on reassignment to '+name)
        frame.values[name]=value

class Function:
    def __init__(self,name,params,body,closure):
        self.name=name
        self.params=params
        self.body=body
        self.closure=closure
