"""Conservative return type inference for a subset of Nobody functions.

The inference does not run any code. In unsupported cases it returns unknown
rather than claiming a particular type or complete program safety.
"""
UNKNOWN = "unknown"
_MAX_PATHS = 64

def _expression(node, values):
    kind = node.kind
    if kind == "number":
        return "float" if "." in node.value else "int"
    if kind == "string":
        return "str"
    if kind == "literal":
        return {"True":"bool","False":"bool","None":"none"}.get(node.value,UNKNOWN)
    if kind == "name":
        return values.get(node.value,UNKNOWN)
    if kind == "list":
        return "list"
    if kind == "unary":
        operand = _expression(node.children[0],values)
        if node.value == "not":
            return "bool"
        return operand if operand in ("int","float") else UNKNOWN
    if kind == "binary":
        a = _expression(node.children[0],values)
        b = _expression(node.children[1],values)
        if node.value in ("==","!=","<","<=",">",">="):
            return "bool" if a != UNKNOWN and b != UNKNOWN else UNKNOWN
        if node.value in ("and","or"):
            return a if a == b else UNKNOWN
        if a == b == "str" and node.value == "+":
            return "str"
        if a in ("int","float") and b in ("int","float"):
            if node.value == "/":
                return "float"
            if node.value == "**":
                return UNKNOWN
            return "float" if "float" in (a,b) else "int"
    return UNKNOWN

def _walk(nodes,start):
    """Return (return_types, possible_fallthrough_environments)."""
    active = [dict(start)]
    returns = []
    for node in nodes:
        future = []
        for env in active:
            kind = node.kind
            if kind in ("let","typed_let"):
                name,annotation=node.value if kind=="typed_let" else (node.value,None)
                if name in env:
                    return [UNKNOWN],[]
                actual=_expression(node.children[0],env)
                next_env=dict(env)
                next_env[name]=annotation or actual
                future.append(next_env)
            elif kind=="assign":
                if node.value not in env:
                    return [UNKNOWN],[]
                next_env=dict(env)
                next_env[node.value]=_expression(node.children[0],env)
                future.append(next_env)
            elif kind=="return":
                returns.append(_expression(node.children[0],env))
            elif kind=="if":
                has_else=any(b.kind=="else" for b in node.children)
                for branch in node.children:
                    body=branch.children if branch.kind=="else" else branch.children[1].children
                    branch_returns,branch_fallthrough=_walk(body,env)
                    returns.extend(branch_returns)
                    future.extend(branch_fallthrough)
                if not has_else:
                    future.append(dict(env))
            elif kind=="function":
                next_env=dict(env)
                next_env[node.value[0]]=UNKNOWN
                future.append(next_env)
            else:
                # Loops, calls with side effects and control flow not handled.
                return [UNKNOWN],[]
        active=future
        if len(active)+len(returns)>_MAX_PATHS:
            return [UNKNOWN],[]
        if not active:
            break
    return returns,active

def infer_function_return(function_node):
    """Return a known type only if every possible exit has the same type."""
    if function_node.kind!="function":
        raise ValueError("Expected Nobody function AST node")
    _,params=function_node.value
    returned,fallthrough=_walk(function_node.children,{p:UNKNOWN for p in params})
    candidates=returned+(["none"] if fallthrough else [])
    if not candidates:
        return UNKNOWN
    return (candidates[0] if candidates[0]!=UNKNOWN and
            all(x==candidates[0] for x in candidates) else UNKNOWN)
