"""Reference catalogue of 249 Google Translate language/script variants.

Registration DOES NOT imply Nobody syntax exists for an entry.
Unimplemented profiles must be rejected, not silently translated into English.
List reference: https://en.wikipedia.org/wiki/Google_Translate#Supported_languages
Retrieved 2026-11-08; attributed list reported as of August 2026.
"""
from dataclasses import dataclass
from pathlib import Path
from .languages import ALIASES

@dataclass(frozen=True)
class LanguageEntry:
    name: str
    status: str
    syntax_code: str | None = None

# Explicitly reviewed mapping between a catalogue variant and a Nobody profile.
# Regional and alternative-script variants are NOT assumed interchangeable.
_MAP = {
    'Slovak':'sk','English':'en','Czech':'cs','German':'de',
    'Spanish':'es','French':'fr','Portuguese (Portugal)':'pt',
    'Russian':'ru','Chinese (Simplified)':'zh','Hindi':'hi',
    'Arabic':'ar','Bengali':'bn','Indonesian':'id','Urdu':'ur',
    'Japanese':'ja','Korean':'ko','Turkish':'tr','Vietnamese':'vi',
    'Tamil':'ta','Italian':'it','Thai':'th','Polish':'pl',
    'Hebrew':'he','Greek':'el','Ukrainian':'uk','Persian':'fa',
    'Swahili':'sw','Dutch':'nl',
    'Bulgarian':'bg','Romanian':'ro','Swedish':'sv',
    'Finnish':'fi','Armenian':'hy','Georgian':'ka',
}
def languages():
    source=Path(__file__).resolve().parent/'data/google_translate_249.txt'
    names=source.read_text(encoding='utf-8').splitlines()
    if len(names)!=249 or len(set(names))!=249:
        raise ValueError('Expected 249 unique language/script variants')
    return tuple(LanguageEntry(name,'experimental' if name in _MAP else 'planned',_MAP.get(name))
                 for name in names)

def status():
    rows=languages()
    return {'reference_variants':len(rows),'active_experimental_profiles':len(ALIASES),
            'planned_reference_variants':sum(x.status=='planned' for x in rows)}
