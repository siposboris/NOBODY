"""Validation and source re-emission for experimental Nobody AST v1.

The loader does not execute programs. This is not a secure code sandbox.
"""
import json
import re
import unicodedata
from .parser import Node, parse_source, BINDING
from .languages import PREFERRED

SCHEMA = "nobody.ast.v1"
EXPR = {"number","string","literal","name","list","index","unary","binary","call"}
STMT = {"let","typed_let","assign","return","while","for","function",
        "break","continue","expression_statement","if"}
ARITIES = {"module":-1,"block":-1,"branch":2,"else":-1,"if":-1,
           "let":1,"typed_let":1,"assign":1,"return":1,"while":2,"for":2,
           "function":-1,"break":0,"continue":0,"expression_statement":1,
           "number":0,"string":0,"literal":0,"name":0,"list":-1,
           "index":2,"unary":1,"binary":2,"call":-1}
class ASTValidationError(ValueError): pass

def valid_name(s):
    return (type(s) is str and 0<len(s)<=256 and s==s.lower() and
            (s[0].isalpha() or s[0]=="_") and
            all(c.isalnum() or c=="_" or unicodedata.category(c).startswith("M")
                for c in s[1:]))

def _node(raw,count,depth=0):
    if depth>128:raise ASTValidationError("AST too deep")
    count[0]+=1
    if count[0]>10000:raise ASTValidationError("Too many AST nodes")
    if type(raw) is not dict or set(raw)!={"kind","line","value","children"}:
        raise ASTValidationError("Unknown or missing AST fields")
    kind,line,value,children=(raw[k] for k in ("kind","line","value","children"))
    if kind not in ARITIES or type(line) is not int or line<1:
        raise ASTValidationError("Invalid kind or line")
    if type(children) is not list or (ARITIES[kind]>=0 and len(children)!=ARITIES[kind]):
        raise ASTValidationError("Invalid child count")
    if kind in {"name","let","assign","for"} and not valid_name(value):
        raise ASTValidationError("Invalid identifier")
    if kind=="typed_let":
        if type(value) not in (list,tuple) or len(value)!=2 or not valid_name(value[0]) or value[1] not in ("int","float","str","bool"):
            raise ASTValidationError("Invalid typed declaration")
    if kind=="function":
        if (type(value) not in (list,tuple) or len(value)!=2 or not valid_name(value[0])
            or type(value[1]) not in (list,tuple) or len(value[1])>128 or
            not all(valid_name(x) for x in value[1]) or len(value[1])!=len(set(value[1]))):
            raise ASTValidationError("Invalid function signature")
    if kind=="binary" and value not in BINDING:raise ASTValidationError("Invalid binary operator")
    if kind=="unary" and value not in ("not","+","-"):raise ASTValidationError("Invalid unary operator")
    if kind=="literal" and value not in ("True","False","None"):raise ASTValidationError("Invalid literal")
    if kind=="number" and (type(value) is not str or not re.fullmatch(r"[0-9][0-9_]*(?:\.[0-9][0-9_]*)?",value)):
        raise ASTValidationError("Invalid numeric spelling")
    if kind=="string" and (type(value) is not str or len(value)>100000 or not value.startswith(('"',"'"))):
        raise ASTValidationError("Invalid string spelling")
    if kind not in {"name","let","typed_let","assign","for","function","binary",
                    "unary","literal","number","string"} and value is not None:
        raise ASTValidationError("Unexpected value")
    nodes=tuple(_node(x,count,depth+1) for x in children)
    def need(i,allowed):
        if nodes[i].kind not in allowed:raise ASTValidationError("Invalid "+kind+" child")
    if kind in ("module","block","else","function") and any(n.kind not in STMT for n in nodes):
        raise ASTValidationError("Invalid block")
    if kind=="if" and (not nodes or any(n.kind not in ("branch","else") for n in nodes)
                       or sum(n.kind=="else" for n in nodes)>1
                       or any(n.kind=="else" for n in nodes[:-1])):
        raise ASTValidationError("Invalid branches")
    if kind=="branch":need(0,EXPR);need(1,{"block"})
    if kind in ("let","typed_let","assign","return","expression_statement"):need(0,EXPR)
    if kind in ("while","for"):need(0,EXPR);need(1,{"block"})
    if kind in ("index","binary"):need(0,EXPR);need(1,EXPR)
    if kind=="unary":need(0,EXPR)
    if kind in ("list","call") and any(n.kind not in EXPR for n in nodes):
        raise ASTValidationError("Invalid expression")
    if kind=="call" and not nodes:raise ASTValidationError("Missing callee")
    if kind=="function":value=(value[0],tuple(value[1]))
    if kind=="typed_let":value=tuple(value)
    return Node(kind,line,value,nodes)

def from_json(document):
    """Validate AST JSON, enforce size/depth/node limits and return immutable nodes."""
    try:
        if type(document) is str:
            if len(document)>2_000_000:raise ASTValidationError("JSON too large")
            document=json.loads(document)
        if (type(document) is not dict or set(document)!={"schema","ast"}
            or document["schema"]!=SCHEMA):
            raise ASTValidationError("Unknown AST schema")
        root=_node(document["ast"],[0])
        if root.kind!="module":raise ASTValidationError("Expected module root")
        return root
    except (ValueError,TypeError,KeyError,RecursionError) as e:
        if isinstance(e,ASTValidationError):raise
        raise ASTValidationError(str(e)) from e

def emit_source(tree,language="sk"):
    """Turn an AST into Nobody source using exactly one chosen syntax profile."""
    if language not in PREFERRED:raise ASTValidationError("Unknown output profile")
    from .ai_ir import node_data
    tree=from_json({"schema":SCHEMA,"ast":node_data(tree)})
    words=PREFERRED[language]
    def kw(k):return words[k]
    def expr(n):
        if n.kind in ("name","number","string"):return n.value
        if n.kind=="literal":return kw(n.value)
        if n.kind=="list":return "["+", ".join(expr(x) for x in n.children)+"]"
        if n.kind=="unary":return "("+(kw("not") if n.value=="not" else n.value)+" "+expr(n.children[0])+")"
        if n.kind=="binary":
            op=kw(n.value) if n.value in ("and","or") else n.value
            return "("+expr(n.children[0])+" "+op+" "+expr(n.children[1])+")"
        if n.kind=="call":return expr(n.children[0])+"("+", ".join(expr(x) for x in n.children[1:])+")"
        if n.kind=="index":return expr(n.children[0])+"["+expr(n.children[1])+"]"
        raise ASTValidationError("Unsupported expression")
    def block(items,depth):
        return "{\n"+"".join(stmt(x,depth+1) for x in items)+"    "*depth+"}"
    def stmt(n,depth=0):
        prefix="    "*depth
        if n.kind=="let":line=kw("let")+" "+n.value+" = "+expr(n.children[0])
        elif n.kind=="typed_let":line=kw("let")+" "+n.value[0]+": "+kw(n.value[1])+" = "+expr(n.children[0])
        elif n.kind=="assign":line=n.value+" = "+expr(n.children[0])
        elif n.kind=="return":line=kw("return")+" "+expr(n.children[0])
        elif n.kind=="expression_statement":line=expr(n.children[0])
        elif n.kind in ("break","continue"):line=kw(n.kind)
        elif n.kind=="function":
            line=kw("def")+" "+n.value[0]+"("+", ".join(n.value[1])+") "+block(n.children,depth)
        elif n.kind in ("while","for"):
            head=(kw("while")+" "+expr(n.children[0]) if n.kind=="while"
                  else kw("for")+" "+n.value+" "+kw("in")+" "+expr(n.children[0]))
            line=head+" "+block(n.children[1].children,depth)
        elif n.kind=="if":
            parts=[]
            for i,b in enumerate(n.children):
                if b.kind=="else":parts.append(kw("else")+" "+block(b.children,depth))
                else:
                    parts.append(kw("if" if i==0 else "elif")+" "+expr(b.children[0])+" "+block(b.children[1].children,depth))
            line=" ".join(parts)
        else:raise ASTValidationError("Unsupported statement")
        return prefix+line+"\n"
    rendered="".join(stmt(n) for n in tree.children)
    parse_source(rendered,language)
    return rendered
