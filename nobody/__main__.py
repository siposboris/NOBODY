"""Nobody standalone experimental frontend command-line entry point.

Only includes tools that are actually committed in this GitHub repository.
Not the full legacy bootstrap interpreter or native compiler.
"""
import argparse
import json
from pathlib import Path
from .catalog import languages, status
from .languages import ALIASES
from .translator import translate
from .ast_vm import execute
from .parser import parse_source
from .bidi import direction_for_language, editor_advice

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
    direction=sub.add_parser('direction')
    direction.add_argument('language')
    info=sub.add_parser('info')
    args=p.parse_args(argv)
    if args.command=='languages':
        if args.json:
            print(json.dumps({'summary':status(),'entries':[vars(x) for x in languages()]},ensure_ascii=False,indent=2))
        else:
            for idx,item in enumerate(languages(),1):
                print(f'{idx:3d}. {item.name}: {item.status} {item.syntax_code or ""}')
    elif args.command=='info':
        print(json.dumps({'name':'Nobody','frontend_version':'0.13-experimental',
                          'reference_entries':249,'syntax_profiles':len(ALIASES),
                          'native_compiler':False,'runtime':'Python bootstrap',
                          'all_249_syntax_implemented':False},indent=2))
    elif args.command=='direction':
        info=direction_for_language(args.language)
        print(info.direction)
        print(editor_advice(args.language))
    else:
        source=Path(args.file).read_text(encoding='utf-8')
        if args.command=='translate':
            print(translate(source,args.from_lang,args.to),end='')
        elif args.command=='parse':
            print(parse_source(source,args.lang))
        elif args.command=='ast-run':
            execute(source,args.lang,max_steps=args.max_steps)

if __name__=='__main__':
    main()
