"""Nobody standalone experimental frontend command-line entry point.

Only includes tools that are actually committed in this GitHub repository.
Not the full legacy bootstrap interpreter or native compiler.
"""
import argparse
import json
import sys
from pathlib import Path
from .catalog import languages, status
from .languages import ALIASES
from .translator import translate
from .ast_vm import execute
from .parser import parse_source
from .bidi import direction_for_language, editor_advice
from .errors import CATALOG,BY_NUMBER,render_exception

def main(argv=None):
    p=argparse.ArgumentParser(prog='nobody')
    sub=p.add_subparsers(dest='command',required=True)
    langs=sub.add_parser('languages',help='Show 249 reference variants and their status')
    langs.add_argument('--json',action='store_true')
    run=sub.add_parser('ast-run',help='Execute experimental Nobody AST subset')
    run.add_argument('file')
    run.add_argument('--lang',required=True)
    run.add_argument('--max-steps',type=int,default=10000)
    tr=sub.add_parser('translate',help='Translate Nobody syntax (Nobody output)')
    tr.add_argument('file')
    tr.add_argument('--from-lang',required=True)
    tr.add_argument('--to',required=True)
    astp=sub.add_parser('parse',help='Display parsed AST')
    astp.add_argument('file')
    astp.add_argument('--lang',required=True)
    astj=sub.add_parser('ast-json',help='Output deterministic versioned JSON AST')
    astj.add_argument('file')
    astj.add_argument('--lang',required=True)
    valid=sub.add_parser('ast-validate',help='Validate Nobody AST v1 JSON (no execution)')
    valid.add_argument('file')
    emitted=sub.add_parser('ast-emit',help='Reconstruct Nobody source from AST JSON')
    emitted.add_argument('file')
    emitted.add_argument('--lang',required=True)
    typecheck=sub.add_parser('type-check',help='Static primitive type analysis without execution')
    typecheck.add_argument('file')
    typecheck.add_argument('--lang',required=True)
    direction=sub.add_parser('direction')
    direction.add_argument('language')
    info=sub.add_parser('info')
    audit=sub.add_parser('audit-languages',help='Audit keyword completeness, lowercase and Unicode safety')
    errors=sub.add_parser('errors',help='List or explain a numbered Nobody error')
    errors.add_argument('number',nargs='?',type=int)
    args=p.parse_args(argv)
    if args.command=='errors':
        entries=([BY_NUMBER[args.number]] if args.number in BY_NUMBER else sorted(CATALOG.values(),key=lambda x:x.number))
        for entry in entries:
            print(f'N{entry.number:04d} [{entry.symbol}]: {entry.title}\n  Príčina: {entry.cause}\n  Oprava: {entry.fix}')
        if args.number is not None and args.number not in BY_NUMBER:raise SystemExit(1)
    elif args.command=='languages':
        if args.json:
            print(json.dumps({'summary':status(),'entries':[vars(x) for x in languages()]},ensure_ascii=False,indent=2))
        else:
            for idx,item in enumerate(languages(),1):
                print(f'{idx:3d}. {item.name}: {item.status} {item.syntax_code or ""}')
    elif args.command=='audit-languages':
        from .profile_validator import profile_summary, audit_profiles
        print(json.dumps(profile_summary(),indent=2))
        for issue in audit_profiles():
            print(issue.language+': '+issue.severity+': '+issue.detail)
    elif args.command=='info':
        print(json.dumps({'name':'Nobody','frontend_version':'0.20-experimental',
                          'reference_entries':249,'syntax_profiles':len(ALIASES),
                          'native_compiler':False,'runtime':'Python bootstrap',
                          'all_249_syntax_implemented':False},indent=2))
    elif args.command=='ast-validate':
        from .ast_format import from_json
        from_json(Path(args.file).read_text(encoding='utf-8'))
        print('Valid nobody.ast.v1')
    elif args.command=='ast-emit':
        from .ast_format import from_json, emit_source
        tree=from_json(Path(args.file).read_text(encoding='utf-8'))
        print(emit_source(tree,args.lang),end='')
    elif args.command=='type-check':
        from .static_types import analyze
        result=analyze(Path(args.file).read_text(encoding='utf-8'),args.lang)
        for item in (*result.diagnostics,*result.warnings):
            print(item.detailed_message)
        if not result.ok: raise SystemExit(1)
        print('OK: no proven primitive type errors; not a full type-safety proof')
    elif args.command=='direction':
        info=direction_for_language(args.language)
        print(info.direction)
        print(editor_advice(args.language))
    else:
        source=Path(args.file).read_text(encoding='utf-8')
        if args.command=='translate':
            print(translate(source,args.from_lang,args.to),end='')
        elif args.command=='ast-json':
            from .ai_ir import to_json
            print(to_json(source,args.lang))
        elif args.command=='parse':
            print(parse_source(source,args.lang))
        elif args.command=='ast-run':
            execute(source,args.lang,max_steps=args.max_steps)

if __name__=='__main__':
    try:
        main()
    except Exception as exc:
        print(render_exception(exc),file=sys.stderr)
        raise SystemExit(1)
