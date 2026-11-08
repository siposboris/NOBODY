"""Native writing directions and bidirectional-source safety for Nobody.

Source and token offsets always remain in LOGICAL order. This module does not
reorder program text. The editor is responsible for visual rendering.
"""
from dataclasses import dataclass
import unicodedata

RTL_LANGUAGES = frozenset({'ar', 'ur'})
BIDI_CONTROLS = frozenset({
    0x061C, 0x200E, 0x200F, 0x202A, 0x202B, 0x202C,
    0x202D, 0x202E, 0x2066, 0x2067, 0x2068, 0x2069,
})


@dataclass(frozen=True)
class SourceDirection:
    language: str
    direction: str
    editor_direction: str
    logical_order: bool = True


def direction_for_language(language: str) -> SourceDirection:
    """Return display metadata; does not modify source bytes."""
    from .languages import ALIASES
    if language not in ALIASES:
        raise ValueError(f'Unsupported Nobody language: {language!r}')
    direction = 'rtl' if language in RTL_LANGUAGES else 'ltr'
    return SourceDirection(language, direction, direction)


def check_bidi_controls(source: str) -> None:
    """Reject invisible bidi overrides/isolation even inside comments and strings."""
    line, column = 1, 1
    for char in source:
        if ord(char) in BIDI_CONTROLS:
            name = unicodedata.name(char, 'UNKNOWN')
            raise ValueError(f'Forbidden bidirectional control U+{ord(char):04X} '
                             f'({name}) at {line}:{column}')
        if char == '\n':
            line, column = line + 1, 1
        else:
            column += 1


def editor_advice(language: str) -> str:
    """Display guidance, not an assertion of universal editor RTL support."""
    info = direction_for_language(language)
    if info.direction == 'rtl':
        return ('Use a UTF-8 source editor with RTL paragraph direction for '
                'Arabic/Urdu text; preserve logical character order and ASCII '
                'operator order. Never insert invisible bidi control characters '
                'into a .nbd source file.')
    return ('Use UTF-8 and normal left-to-right code layout; retain the '
            'selected language native keyword spellings.')
