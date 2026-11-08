"""Experimental Nobody AST parser, currently bootstrapped in Python.

Only brace-syntax subset is supported. It does not execute programs.
"""
from dataclasses import dataclass
from .lexer import scan
from .languages import ALIASES


class ParseError(ValueError):
    pass


@dataclass(frozen=True)
class Node:
    kind: str
    line: int
    value: object = None
    children: tuple = ()


PRECEDENCE = {'or': 1, 'and': 2, '==': 3, '!=': 3, '<': 3, '<=': 3,
              '>': 3, '>=': 3, '+': 4, '-': 4, '*': 5, '/': 5,
              '//': 5, '%': 5, '**': 7}


class Parser:
    def __init__(self, source, language='sk'):
        if language not in ALIASES:
            raise ParseError(f'Unsupported syntax language {language!r}')
        self.aliases = ALIASES[language]
        tokens = [t for t in scan(source) if t.kind != 'comment']
        foreign = set()
        for lang, mapping in ALIASES.items():
            if lang != language:
                foreign.update(word for word in mapping if word not in self.aliases)
        for t in tokens:
            if t.kind == 'name' and t.value in foreign:
                raise ParseError(f'Foreign keyword {t.value!r} at {t.line}:{t.column}')
        self.tokens = []
        i = 0
        while i < len(tokens):
            t = tokens[i]
            if i + 1 < len(tokens):
                nxt = tokens[i + 1]
                value = t.value + nxt.value
                if t.end == nxt.start and value in ('==', '!=', '<=', '>=', '**', '//'):
                    self.tokens.append(type(t)('symbol', value, t.line, t.column, t.start, nxt.end))
                    i += 2
                    continue
            self.tokens.append(t)
            i += 1
        self.index = 0

    def peek(self):
        return self.tokens[self.index] if self.index < len(self.tokens) else None

    def at(self, value):
        return self.peek() is not None and self.peek().value == value

    def take(self):
        token = self.peek()
        if token is None:
            raise ParseError('Unexpected end of input')
        self.index += 1
        return token

    def accept(self, value):
        if self.at(value):
            return self.take()
        return None

    def error(self, message):
        token = self.peek()
        where = f'{token.line}:{token.column}' if token else 'end of input'
        return ParseError(f'{message} at {where}')

    def expect(self, value):
        token = self.accept(value)
        if token is None:
            raise self.error(f'Expected {value!r}')
        return token

    def canonical(self, token):
        return self.aliases.get(token.value, token.value)

    def keyword(self, canonical):
        token = self.peek()
        if token and token.kind == 'name' and self.canonical(token) == canonical:
            return self.take()
        return None

    def identifier(self):
        token = self.take()
        if token.kind != 'name' or token.value in self.aliases:
            raise ParseError(f'Expected identifier at {token.line}:{token.column}')
        return token

    def parse(self):
        body = []
        while self.peek() is not None:
            if self.at('}'):
                raise self.error('Unexpected closing brace')
            body.append(self.statement())
        return Node('module', 1, children=tuple(body))

    def block(self):
        self.expect('{')
        body = []
        while not self.at('}'):
            if self.peek() is None:
                raise self.error('Unclosed block')
            body.append(self.statement())
        self.expect('}')
        return tuple(body)

    def statement(self):
        if token := self.keyword('let'):
            name = self.identifier()
            self.expect('=')
            expr = self.expression()
            self.accept(';')
            return Node('let', token.line, name.value, (expr,))
        if token := self.keyword('return'):
            expr = self.expression()
            self.accept(';')
            return Node('return', token.line, children=(expr,))
        if token := self.keyword('if'):
            test = self.expression()
            branches = [Node('branch', token.line, children=(test, Node('block', token.line, children=self.block())))]
            while nxt := self.keyword('elif'):
                branches.append(Node('branch', nxt.line, children=(self.expression(), Node('block', nxt.line, children=self.block()))))
            if nxt := self.keyword('else'):
                branches.append(Node('else', nxt.line, children=self.block()))
            return Node('if', token.line, children=tuple(branches))
        if token := self.keyword('while'):
            test = self.expression()
            return Node('while', token.line, children=(test, Node('block', token.line, children=self.block())))
        if token := self.keyword('for'):
            name = self.identifier()
            if not self.keyword('in'):
                raise self.error('Expected loop membership keyword')
            seq = self.expression()
            return Node('for', token.line, name.value, (seq, Node('block', token.line, children=self.block())))
        if token := self.keyword('def'):
            name = self.identifier()
            self.expect('(')
            params = []
            if not self.at(')'):
                while True:
                    params.append(self.identifier().value)
                    if not self.accept(','):
                        break
            self.expect(')')
            return Node('function', token.line, (name.value, tuple(params)), self.block())
        if token := self.keyword('break'):
            self.accept(';')
            return Node('break', token.line)
        if token := self.keyword('continue'):
            self.accept(';')
            return Node('continue', token.line)
        expr = self.expression()
        self.accept(';')
        return Node('expression_statement', expr.line, children=(expr,))

    def expression(self, min_bp=0):
        token = self.take()
        word = self.canonical(token)
        if token.kind == 'number':
            left = Node('number', token.line, token.value)
        elif token.kind == 'string':
            left = Node('string', token.line, token.value)
        elif token.kind == 'name' and word in ('True', 'False', 'None'):
            left = Node('literal', token.line, word)
        elif token.kind == 'name' and word == 'not':
            left = Node('unary', token.line, word, (self.expression(6),))
        elif token.value in ('-', '+'):
            left = Node('unary', token.line, token.value, (self.expression(6),))
        elif token.value == '(':
            left = self.expression()
            self.expect(')')
        elif token.value == '[':
            elements = []
            if not self.at(']'):
                while True:
                    elements.append(self.expression())
                    if not self.accept(','):
                        break
            self.expect(']')
            left = Node('list', token.line, children=tuple(elements))
        elif token.kind == 'name' and word not in (
            'if', 'else', 'elif', 'while', 'for', 'def', 'return', 'break',
            'continue', 'let', 'in', 'and', 'or'):
            left = Node('name', token.line, token.value)
        else:
            raise ParseError(f'Expected expression at {token.line}:{token.column}')

        while self.peek() is not None:
            if self.at('(') and 9 >= min_bp:
                self.take()
                args = []
                if not self.at(')'):
                    while True:
                        args.append(self.expression())
                        if not self.accept(','):
                            break
                self.expect(')')
                left = Node('call', left.line, children=(left, *args))
                continue
            if self.at('[') and 9 >= min_bp:
                self.take()
                index = self.expression()
                self.expect(']')
                left = Node('index', left.line, children=(left, index))
                continue
            operator = self.canonical(self.peek())
            bp = PRECEDENCE.get(operator)
            if bp is None or bp < min_bp:
                break
            op = self.take()
            right = self.expression(bp if operator == '**' else bp + 1)
            left = Node('binary', op.line, operator, (left, right))
        return left


def parse_source(source, language='sk'):
    """Produce an AST for the supported Nobody source subset."""
    return Parser(source, language).parse()
