"""Experimental AST evaluator for Nobody's brace syntax subset.

No Nobody-to-Python source generation is involved. The evaluator itself is
still written in Python, requires Python at runtime and IS NOT A SANDBOX.
"""
import ast
import operator
from .parser import parse_source
from .languages import ALIASES
from .scopes import Scope, Function, ExecutionError

class _Return(Exception):
    def __init__(self,value): self.value=value
class _Break(Exception): pass
class _Continue(Exception): pass

OPS={'+':operator.add,'-':operator.sub,'*':operator.mul,
     '/':operator.truediv,'//':operator.floordiv,'%':operator.mod,
     '**':operator.pow,'==':operator.eq,'!=':operator.ne,
     '<':operator.lt,'<=':operator.le,'>':operator.gt,'>=':operator.ge}
BUILTINS={'range':range,'len':len,'sum':sum,'min':min,'max':max,
          'abs':abs,'round':round,'int':int,'float':float,'str':str,'bool':bool}

class Machine:
    def __init__(self,language='sk',max_steps=10000,output=None):
        self.language=language
        self.aliases=ALIASES[language]
        self.steps=0
        self.max_steps=max_steps
        self.call_depth=0
        self.max_call_depth=128
        self.builtins=dict(BUILTINS)
        self.builtins['print']=output or print
        for word,canonical in self.aliases.items():
            if canonical in self.builtins:
                self.builtins[word]=self.builtins[canonical]

    def tick(self):
        self.steps+=1
        if self.steps>self.max_steps:
            raise ExecutionError('Nobody AST step limit exceeded')

    def expr(self,node,env):
        self.tick()
        if node.kind=='number':
            spelling=node.value.replace('_','')
            if not spelling.replace('.','',1).isdigit():
                raise ExecutionError('Unsupported number literal')
            return float(spelling) if '.' in spelling else int(spelling)
        if node.kind=='string':
            return ast.literal_eval(node.value)
        if node.kind=='literal':
            return {'True':True,'False':False,'None':None}[node.value]
        if node.kind=='name':
            owner=env.owner(node.value)
            if owner is not None:return owner.values[node.value]
            if node.value in self.builtins:return self.builtins[node.value]
            raise ExecutionError('Unknown name: '+node.value)
        if node.kind=='list':
            return [self.expr(n,env) for n in node.children]
        if node.kind=='index':
            return self.expr(node.children[0],env)[self.expr(node.children[1],env)]
        if node.kind=='unary':
            value=self.expr(node.children[0],env)
            return not value if node.value=='not' else (-value if node.value=='-' else +value)
        if node.kind=='binary':
            left=self.expr(node.children[0],env)
            if node.value=='and':return left and self.expr(node.children[1],env)
            if node.value=='or':return left or self.expr(node.children[1],env)
            if node.value not in OPS:raise ExecutionError('Unsupported operator')
            right=self.expr(node.children[1],env)
            if node.value=='**' and isinstance(right,(int,float)) and abs(right)>100:
                raise ExecutionError('Exponent exceeds limit')
            return OPS[node.value](left,right)
        if node.kind=='call':
            callee=self.expr(node.children[0],env)
            args=[self.expr(n,env) for n in node.children[1:]]
            if isinstance(callee,Function):
                if len(args)!=len(callee.params):raise ExecutionError('Wrong argument count')
                if self.call_depth>=self.max_call_depth:
                    raise ExecutionError('Maximum function call depth exceeded')
                frame=Scope(callee.closure)
                for name,arg in zip(callee.params,args):frame.declare(name,arg)
                self.call_depth+=1
                try:
                    try:self.block(callee.body,frame,in_function=True)
                    except _Return as returned:return returned.value
                    return None
                finally:self.call_depth-=1
            if callee in self.builtins.values():return callee(*args)
            raise ExecutionError('Call not permitted')
        raise ExecutionError('Unsupported expression: '+node.kind)

    def block(self,nodes,env,in_loop=False,in_function=False):
        for node in nodes:
            self.tick()
            if node.kind in ('let','typed_let'):
                name,annotation=node.value if node.kind=='typed_let' else (node.value,None)
                kind={'int':int,'float':float,'str':str,'bool':bool}[annotation] if annotation is not None else None
                env.declare(name,self.expr(node.children[0],env),kind)
            elif node.kind=='assign':
                env.assign(node.value,self.expr(node.children[0],env))
            elif node.kind=='expression_statement':
                self.expr(node.children[0],env)
            elif node.kind=='function':
                env.declare(node.value[0],Function(node.value[0],node.value[1],node.children,env))
            elif node.kind=='if':
                for branch in node.children:
                    if branch.kind=='else':
                        self.block(branch.children,Scope(env),in_loop,in_function)
                        break
                    if self.expr(branch.children[0],env):
                        self.block(branch.children[1].children,Scope(env),in_loop,in_function)
                        break
            elif node.kind=='while':
                while self.expr(node.children[0],env):
                    try:self.block(node.children[1].children,Scope(env),True,in_function)
                    except _Continue:continue
                    except _Break:break
            elif node.kind=='for':
                for value in self.expr(node.children[0],env):
                    frame=Scope(env)
                    frame.declare(node.value,value)
                    try:self.block(node.children[1].children,frame,True,in_function)
                    except _Continue:continue
                    except _Break:break
            elif node.kind=='return':
                if not in_function:raise ExecutionError('Return outside function')
                raise _Return(self.expr(node.children[0],env))
            elif node.kind=='break':
                if not in_loop:raise ExecutionError('Break outside loop')
                raise _Break()
            elif node.kind=='continue':
                if not in_loop:raise ExecutionError('Continue outside loop')
                raise _Continue()
            else:
                raise ExecutionError('Unsupported AST node: '+node.kind)

def execute(source,language='sk',*,max_steps=10000,output=None):
    machine=Machine(language,max_steps,output)
    tree=parse_source(source,language)
    frame=Scope()
    machine.block(tree.children,frame)
    return {'environment':dict(frame.values),'steps':machine.steps,'ast':tree}
