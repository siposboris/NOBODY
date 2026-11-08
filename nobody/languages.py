"""Spelling tables for syntax in exactly one human language per file.

Canonical values are internal compiler concepts, not mixed-language syntax.
"""
ALIASES = {
    'sk': {
        'ak':'if','inak':'else','inak_ak':'elif','pokial':'while','kym':'while',
        'pre':'for','v':'in','funkcia':'def','vrat':'return','pravda':'True',
        'nepravda':'False','nic':'None','a_zaroven':'and','alebo':'or','nie':'not',
        'prerus':'break','pokracuj':'continue','nech':'let',
        'vypis':'print','rozsah':'range','dlzka':'len','sucet':'sum',
        'minimum':'min','maximum':'max','absolutne':'abs','zaokruhli':'round',
        'cele':'int','desatinne':'float','text':'str','logicke':'bool'
    },
    'en': {
        'if':'if','else':'else','elif':'elif','while':'while','for':'for','in':'in',
        'function':'def','def':'def','return':'return','true':'True','false':'False',
        'none':'None','and':'and','or':'or','not':'not','break':'break',
        'continue':'continue','let':'let','print':'print','range':'range',
        'len':'len','sum':'sum','min':'min','max':'max','abs':'abs',
        'round':'round','int':'int','float':'float','str':'str','bool':'bool'
    },
    'cs': {
        'pokud':'if','jinak':'else','jinak_pokud':'elif','dokud':'while',
        'pro':'for','v':'in','funkce':'def','vrat':'return','pravda':'True',
        'nepravda':'False','nic':'None','a_zaroven':'and','nebo':'or','ne':'not',
        'prerus':'break','pokracuj':'continue','necht':'let','vypis':'print',
        'rozsah':'range','delka':'len','soucet':'sum','minimum':'min','maximum':'max'
    },
    'de': {
        'wenn':'if','sonst':'else','sonst_wenn':'elif','solange':'while',
        'fur':'for','in':'in','funktion':'def','zuruck':'return','wahr':'True',
        'falsch':'False','nichts':'None','und':'and','oder':'or','nicht':'not',
        'abbruch':'break','weiter':'continue','sei':'let','ausgeben':'print',
        'bereich':'range','lange':'len','summe':'sum','minimum':'min','maximum':'max'
    }
}
CANONICAL = {'if','else','elif','while','for','in','def','return','True','False','None',
             'and','or','not','break','continue','let','print','range','len','sum',
             'min','max','abs','round','int','float','str','bool'}
PREFERRED = {
    'sk': {
        'if':'ak','else':'inak','elif':'inak_ak','while':'pokial','for':'pre','in':'v',
        'def':'funkcia','return':'vrat','True':'pravda','False':'nepravda',
        'None':'nic','and':'a_zaroven','or':'alebo','not':'nie','break':'prerus',
        'continue':'pokracuj','let':'nech','print':'vypis','range':'rozsah',
        'len':'dlzka','sum':'sucet','min':'minimum','max':'maximum',
        'abs':'absolutne','round':'zaokruhli','int':'cele',
        'float':'desatinne','str':'text','bool':'logicke'
    }
}
for language in ('en','cs','de'):
    preferred = {}
    for spelling, canonical in ALIASES[language].items():
        preferred.setdefault(canonical, spelling)
    PREFERRED[language] = preferred
