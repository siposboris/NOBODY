"""Versioned machine-readable AST, useful for agents and code tools.

This is a deterministic serialization of Nobody's syntax tree, not a new
AI cognition substrate, model weights, bytecode or native compilation.
"""
import json
from .parser import parse_source

SCHEMA = 'nobody.ast.v1'

def _node(node):
    value = list(node.value) if isinstance(node.value,tuple) else node.value
    return {'kind':node.kind,'line':node.line,'value':value,
            'children':[_node(child) for child in node.children]}

def node_data(node):
    """Convert an immutable AST node into the public AST v1 data mapping.

    Shared by JSON export and source re-emission, without evaluating source.
    """
    return _node(node)

def to_json(source,language='sk'):
    tree=parse_source(source,language)
    return json.dumps({'schema':SCHEMA,'ast':node_data(tree)},ensure_ascii=False,
                      sort_keys=True,separators=(',',':'))
