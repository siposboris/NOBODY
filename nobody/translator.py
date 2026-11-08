"""Unicode-aware Nobody-to-Nobody source translator; no Python output.

Only recognized syntax words and built-in names are changed.
Strings, comments, spaces and user identifiers are preserved as written.
"""
from .lexer import scan, LexError
from .languages import ALIASES, PREFERRED, NATIVE_SCRIPT_PROFILES


class SyntaxLanguageError(ValueError):
    pass


def validate_language(source: str, language: str) -> None:
    if language not in ALIASES:
        raise SyntaxLanguageError(f"Unsupported syntax language: {language!r}")
    current = ALIASES[language]
    foreign = {word for profile, words in ALIASES.items()
               if profile != language for word in words if word not in current}
    try:
        for token in scan(source):
            if token.kind != 'name':
                continue
            if language not in NATIVE_SCRIPT_PROFILES and not token.value.isascii():
                raise SyntaxLanguageError(
                    f"Non-ASCII identifier {token.value!r} in {language} source at {token.line}:{token.column}")
            if token.value in foreign:
                raise SyntaxLanguageError(
                    f"Foreign syntax keyword {token.value!r} in {language} source at {token.line}:{token.column}")
    except LexError as exc:
        raise SyntaxLanguageError(str(exc)) from exc


def _replace_names(source: str, mapping: dict[str, str]) -> str:
    chunks = []
    cursor = 0
    for token in scan(source):
        if token.kind == 'name' and token.value in mapping:
            chunks.append(source[cursor:token.start])
            chunks.append(mapping[token.value])
            cursor = token.end
    chunks.append(source[cursor:])
    return ''.join(chunks)


def translate(source: str, from_lang: str, to_lang: str) -> str:
    """Translate code between two supported syntax profiles, preserving literals."""
    if to_lang not in PREFERRED:
        raise SyntaxLanguageError(f"Unsupported output language: {to_lang!r}")
    validate_language(source, from_lang)
    mapping = {word: PREFERRED[to_lang][canonical]
               for word, canonical in ALIASES[from_lang].items()}
    result = _replace_names(source, mapping)
    validate_language(result, to_lang)
    return result
