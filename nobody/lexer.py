"""Nobody experimental lexer: lower-case ASCII names, Unicode strings/comments.

This is not yet a complete grammar parser.
"""
from dataclasses import dataclass
import unicodedata


class LexError(ValueError):
    pass


@dataclass(frozen=True)
class Token:
    kind: str
    value: str
    line: int
    column: int
    start: int
    end: int


def scan(source: str) -> list[Token]:
    result = []
    pos, line, col, length = 0, 1, 1, len(source)

    def advance(end):
        nonlocal pos, line, col
        frag = source[pos:end]
        count = frag.count('\n')
        if count:
            line += count
            col = len(frag.rsplit('\n', 1)[-1]) + 1
        else:
            col += len(frag)
        pos = end

    while pos < length:
        c = source[pos]
        if c in ' \t\r\n':
            advance(pos + 1)
            continue
        start, tok_line, tok_col = pos, line, col
        if c == '#':
            end = source.find('\n', pos)
            if end == -1:
                end = length
            result.append(Token('comment', source[pos:end], line, col, pos, end))
            advance(end)
            continue
        if c in ('"', "'"):
            quote = c
            triple = source.startswith(quote * 3, pos)
            opener = 3 if triple else 1
            j = pos + opener
            closing = quote * opener
            while j < length:
                if source[j] == '\\':
                    j += 2
                    continue
                if source.startswith(closing, j):
                    j += opener
                    break
                if source[j] == '\n' and not triple:
                    raise LexError(f'Unterminated string at {tok_line}:{tok_col}')
                j += 1
            else:
                raise LexError(f'Unterminated string at {tok_line}:{tok_col}')
            result.append(Token('string', source[start:j], tok_line, tok_col, start, j))
            advance(j)
            continue
        if c.isalpha() or c == '_':
            end = pos + 1
            while end < length and (source[end].isalnum() or source[end] == '_' or unicodedata.category(source[end]).startswith('M')):
                end += 1
            word = source[start:end]
            if word != word.lower():
                raise LexError(f'Identifier/keyword {word!r} at {tok_line}:{tok_col} must be lowercase ascii')
            result.append(Token('name', word, tok_line, tok_col, start, end))
            advance(end)
            continue
        if unicodedata.category(c).startswith('M'):
            raise LexError(f'Unexpected combining mark at {tok_line}:{tok_col}')
        if c.isdigit():
            end = pos + 1
            while end < length and (source[end].isascii() and (source[end].isalnum() or source[end] in '._')):
                end += 1
            result.append(Token('number', source[start:end], tok_line, tok_col, start, end))
            advance(end)
            continue
        if not c.isascii():
            raise LexError(f'Unexpected non-ascii character at {tok_line}:{tok_col} outside string/comment')
        result.append(Token('symbol', c, tok_line, tok_col, start, start + 1))
        advance(pos + 1)
    return result
