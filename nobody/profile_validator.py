"""Lexical audit of Nobody language dictionaries.

This cannot replace native-speaker validation of correct terminology.
"""
from dataclasses import dataclass
import unicodedata
from .languages import ALIASES, CANONICAL
from .bidi import BIDI_CONTROLS

@dataclass(frozen=True)
class ProfileIssue:
    language: str
    severity: str
    detail: str

def audit_profiles():
    issues=[]
    required=set(CANONICAL)
    for language, mapping in ALIASES.items():
        missing=required-set(mapping.values())
        if missing:
            issues.append(ProfileIssue(language,'error',
                'Missing semantic tokens: '+', '.join(sorted(missing))))
        if len(mapping)!=len(set(mapping.values())):
            issues.append(ProfileIssue(language,'info',
                'Some canonical tokens have multiple aliases'))
        for word in mapping:
            if not word.isidentifier():
                issues.append(ProfileIssue(language,'error','Invalid keyword identifier '+word))
            if word!=word.lower():
                issues.append(ProfileIssue(language,'error','Uppercase keyword '+word))
            if unicodedata.normalize('NFC',word)!=word:
                issues.append(ProfileIssue(language,'warning','Non-NFC keyword '+word))
            if any(ord(c) in BIDI_CONTROLS for c in word):
                issues.append(ProfileIssue(language,'error','Bidi formatting control in keyword'))
    return tuple(issues)

def profile_summary():
    issues=audit_profiles()
    return {'profiles':len(ALIASES),
            'errors':sum(x.severity=='error' for x in issues),
            'warnings':sum(x.severity=='warning' for x in issues),
            'notes':sum(x.severity=='info' for x in issues)}
