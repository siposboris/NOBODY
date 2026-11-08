"""Conservative Nobody AST static checker: only provable primitive errors.

A successful check is not a proof of whole-program type safety.
"""
from dataclasses import dataclass
from .parser import parse_source
from .languages import ALIASES
from .errors import definition, format_diagnostic
from .return_types import infer_function_return

UNKNOWN="unknown"
@dataclass(frozen=True)
class Diagnostic:
    line:int
    code:str
    message:str
    @property
    def number(self):return definition(self.code).number
    @property
    def detailed_message(self):return format_diagnostic(self.code,self.message,self.line)

@dataclass(frozen=True)
class Analysis:
    diagnostics:tuple
    warnings:tuple
    @property
    def ok(self):return not self.diagnostics

class Environment:
    def __init__(self,parent=None):
        self.parent=parent
        self.vars={}
        self.signatures={}
        self.return_types={}
    def find(self,name):
        if name in self.vars:return self
        return self.parent.find(name) if self.parent else None
    def signature(self,name):
        owner=self.find(name)
        return owner.signatures.get(name) if owner else None
    def function_return(self,name):
        owner=self.find(name)
        return owner.return_types.get(name,UNKNOWN) if owner else UNKNOWN
    def visible_frames(self):
        frames=[]
        current=self
        while current:
            frames.append(current)
            current=current.parent
        return frames

class Checker:
    def __init__(self,language):
        self.language=language
        self.errors=[]
        self.warnings=[]
        self.builtins={name for name,semantic in ALIASES[language].items()
                       if semantic in ("print","len","range","sum","min","max","abs","round","int","float","str","bool")}
    def problem(self,node,code,message):
        self.errors.append(Diagnostic(node.line,code,message))
    def expr(self,n,env):
        k=n.kind
        if k=="number":return "float" if "." in n.value else "int"
        if k=="string":return "str"
        if k=="literal":return {"True":"bool","False":"bool","None":"none"}.get(n.value,UNKNOWN)
        if k=="name":
            owner=env.find(n.value)
            if owner:return owner.vars[n.value][1]
            if n.value in self.builtins:return "function"
            self.warnings.append(Diagnostic(n.line,"W_DYNAMIC_NAME",f"Unknown type for {n.value!r}"))
            return UNKNOWN
        if k=="list":
            for child in n.children:self.expr(child,env)
            return "list"
        if k=="index":
            self.expr(n.children[0],env);self.expr(n.children[1],env)
            return UNKNOWN
        if k=="unary":
            value=self.expr(n.children[0],env)
            if n.value=="not":return "bool"
            if value not in ("int","float",UNKNOWN):
                self.problem(n,"E_UNARY",f"Numeric unary operator cannot use {value}")
            return value if value in ("int","float") else UNKNOWN
        if k=="binary":
            left=self.expr(n.children[0],env)
            right=self.expr(n.children[1],env)
            op=n.value
            if op in ("and","or"):return UNKNOWN if UNKNOWN in (left,right) else right
            if op in ("==","!=","<","<=",">",">="):return "bool"
            if UNKNOWN in (left,right):return UNKNOWN
            if op=="+" and left==right=="str":return "str"
            if left in ("int","float") and right in ("int","float"):
                if op=="/":return "float"
                if op=="//":return "float" if "float" in (left,right) else "int"
                if op=="**":return UNKNOWN
                return "float" if "float" in (left,right) else "int"
            self.problem(n,"E_BINARY",f"Invalid operator {op!r} for {left} and {right}")
            return UNKNOWN
        if k=="call":
            for child in n.children:self.expr(child,env)
            callee=n.children[0]
            if callee.kind=="name":
                count=env.signature(callee.value)
                if count is not None and len(n.children)-1!=count:
                    self.problem(n,"E_ARITY",f"Function {callee.value!r} expects {count} arguments, got {len(n.children)-1}")
                    return UNKNOWN
                if count is not None:
                    return env.function_return(callee.value)
                canonical=ALIASES[self.language].get(callee.value)
                if canonical in ("int","float","str","bool"):return canonical
                if canonical=="len":return "int"
                if canonical=="range":return "range"
            return UNKNOWN
        return UNKNOWN
    def walk(self,nodes,env):
        for n in nodes:
            k=n.kind
            if k in ("let","typed_let"):
                name,annotation=n.value if k=="typed_let" else (n.value,None)
                actual=self.expr(n.children[0],env)
                if name in env.vars:self.problem(n,"E_DUPLICATE",f"Duplicate declaration {name!r}")
                if annotation and actual!=UNKNOWN and annotation!=actual:
                    self.problem(n,"E_TYPE",f"{name!r} is {annotation}, expression is {actual}")
                env.vars[name]=(annotation,annotation or actual)
            elif k=="assign":
                actual=self.expr(n.children[0],env)
                owner=env.find(n.value)
                if owner is None:
                    self.problem(n,"E_UNDECLARED",f"Cannot assign undeclared variable {n.value!r}")
                else:
                    annotation,inferred=owner.vars[n.value]
                    if annotation and actual!=UNKNOWN and annotation!=actual:
                        self.problem(n,"E_TYPE",f"Cannot assign {actual} to {annotation} variable {n.value!r}")
                    if annotation is None:
                        owner.vars[n.value]=(None,actual)
            elif k=="function":
                name,params=n.value
                if name in env.vars:self.problem(n,"E_DUPLICATE",f"Duplicate function {name!r}")
                env.vars[name]=(None,"function")
                env.signatures[name]=len(params)
                env.return_types[name]=infer_function_return(n)
                local=Environment(env)
                for param in params:local.vars[param]=(None,UNKNOWN)
                self.walk(n.children,local)
            elif k=="if":
                original=self.snapshot(env)
                outcomes=[]
                for branch in n.children:
                    self.restore(original)
                    if branch.kind=="else":
                        self.walk(branch.children,Environment(env))
                    else:
                        self.expr(branch.children[0],env)
                        self.walk(branch.children[1].children,Environment(env))
                    outcomes.append(self.snapshot(env))
                if not any(branch.kind=="else" for branch in n.children):
                    outcomes.append(original)
                self.merge(original,outcomes)
            elif k=="while":
                self.expr(n.children[0],env)
                original=self.snapshot(env)
                self.walk(n.children[1].children,Environment(env))
                body=self.snapshot(env)
                self.merge(original,[original,body])
            elif k=="for":
                self.expr(n.children[0],env)
                original=self.snapshot(env)
                local=Environment(env)
                local.vars[n.value]=(None,UNKNOWN)
                self.walk(n.children[1].children,local)
                body=self.snapshot(env)
                self.merge(original,[original,body])
            elif k in ("return","expression_statement"):
                self.expr(n.children[0],env)
    @staticmethod
    def snapshot(env):
        return [(frame,dict(frame.vars)) for frame in env.visible_frames()]

    @staticmethod
    def restore(snapshot):
        for frame,variables in snapshot:
            frame.vars=dict(variables)

    @staticmethod
    def merge(before,outcomes):
        # A type survives only when all possible control-flow paths agree.
        for position,(frame,original) in enumerate(before):
            for name,(annotation,inferred) in original.items():
                if annotation is not None:
                    frame.vars[name]=(annotation,annotation)
                else:
                    variants=[case[position][1][name][1] for case in outcomes]
                    frame.vars[name]=(None,variants[0] if all(x==variants[0] for x in variants) else UNKNOWN)

    def check(self,tree):
        self.walk(tree.children,Environment())
        return Analysis(tuple(self.errors),tuple(self.warnings))

def analyze(source,language="sk"):
    """Parse and diagnose a conservative subset without executing the code."""
    return Checker(language).check(parse_source(source,language))
