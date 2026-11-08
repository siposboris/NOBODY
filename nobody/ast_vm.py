"""Experimental AST evaluator for Nobody's brace syntax subset.

No Nobody-to-Python source generation is involved. The evaluator itself is
still written in Python, requires Python at runtime and IS NOT A SANDBOX.
"""
import ast
import operator
from .parser import parse_source
from .languages import ALIASES

class ExecutionError(RuntimeError): pass
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
        self.functions={}
        self.declared_types={}
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
            if node.value in env:return env[node.value]
            if node.value in self.builtins:return self.builtins[node.value]
            if node.value in self.functions:return ('function',node.value)
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
            if isinstance(callee,tuple) and callee[0]=='function':
                params,body=self.functions[callee[1]]
                if len(args)!=len(params:raise ExecutionError('Wrong argument count')
                try:self.block(body,dict(zip(params,args)),in_function=True)
                except _Return as returned:return returned.value
                return None
            if callee in self.builtins.values():return callee(*args)
            raise ExecutionError('Call not permitted')
        raise ExecutionError('Unsupported expression: '+node.kind)

    def block(self,nodes,env,in_loop=False,in_function=False):
        for node in nodes:
            self.tick()
            if node.kind in ('let','typed_let'):
                name, annotation = node.value if node.kind=='typed_let' else (node.value,None)
                if name in env:raise ExecutionError('Duplicate declaration')
                value=self.expr(node.children[0],env)
                if annotation is not None and type(value) is not {'int':int,'float':float,'str':str,'bool':bool}[annotation]:
                    raise ExecutionError('Type mismatch for '+name)
                env[name]=value
                if annotation is not None:
                    self.declared_types.setdefault(id(env),{})[name]={'int':int,'float':float,'str':str,'bool':bool}[annotation]
            elif node.kind=='assign':
                if node.value not in env:
                    raise ExecutionError('Assignment to undeclared variable '+node.value)
                value=self.expr(node.children[0],env)
                expected=self.declared_types.get(id(env),{}).get(node.value)
                if expected is not None and type(value) is not expected:
                    raise ExecutionError('Type mismatch on reassignment to '+node.value)
                env[node.value]=value
            elif node.kind=='expression_statement':
                self.expr(node.children[0],env)
            elif node.kind=='function':
                self.functions[node.value[0]]=(node.value[1],node.children)
            elif node.kind=='if':
                for branch in node.children:
                    if branch.kind=='else':
                        self.block(branch.children,env,in_loop,in_function)
                        break
                    if self.expr(branch.children[0],env):
                        self.block(branch.children[1].children,env,in_loop,in_function)
                        break
            elif node.kind=='while':
                while self.expr(node.children[0],env):
                    try:self.block(node.children[1].children,env,True,in_function)
                    except _Continue:continue
                    except _Break:break
            elif node.kind=='for':
                for value in self.expr(node.children[0],env):
                    env[node.value]=value
                    try:self.block(node.children[1].children,env,True,in_function)
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
    frame={}
    machine.block(tree.children,frame)
    return {'environment':frame,'steps':machine.steps,'ast':tree}
