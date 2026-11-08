"""Nobody AST v1 JSON loader and Nobody source regeneration tests."""
import json
import unittest
from nobody.parser import parse_source
from nobody.ai_ir import to_json
from nobody.ast_format import from_json, emit_source, ASTValidationError
from nobody.translator import translate
from nobody.ast_vm import execute

SOURCE = """nech n: cele = 2
funkcia dvoj(x) { vrat x * 2 }
pokial n < 4 { n = n + 1 }
ak n > 2 { vypis(dvoj(n)) } inak { vypis(0) }
"""

class ASTFormatTests(unittest.TestCase):
    def test_multilingual_roundtrip_and_execution(self):
        def shape(n):return (n.kind,n.value,tuple(shape(c) for c in n.children))
        for lang in ("sk","en","ai","ar","ja","ne","fa","de"):
            with self.subTest(language=lang):
                original=translate(SOURCE,"sk",lang)
                tree=from_json(to_json(original,lang))
                rendered=emit_source(tree,lang)
                self.assertEqual(shape(parse_source(rendered,lang)),shape(parse_source(original,lang)))
                result=[]
                execute(rendered,lang,output=lambda *args:result.append(args))
                self.assertEqual(result,[(8,)])

    def test_deterministic_output(self):
        tree=from_json(to_json(SOURCE))
        self.assertEqual(emit_source(tree),emit_source(tree))

    def test_schema_rejected(self):
        payload=json.loads(to_json(SOURCE))
        payload["schema"]="nobody.ast.v999"
        with self.assertRaises(ASTValidationError):from_json(payload)

    def test_reject_unknown_fields_and_ast_nodes(self):
        for mode in ("extra","kind","children"):
            with self.subTest(mode=mode):
                payload=json.loads(to_json(SOURCE))
                if mode=="extra":payload["ast"]["hidden"]="data"
                if mode=="kind":payload["ast"]["kind"]="eval_python"
                if mode=="children":payload["ast"]["children"]={}
                with self.assertRaises(ASTValidationError):from_json(payload)

    def test_reject_invalid_annotations(self):
        for val in (["n","unknown"],["n","int",1],["bad name","int"]):
            node={"kind":"typed_let","line":1,"value":val,"children":[
                {"kind":"number","line":1,"value":"1","children":[]}]}
            payload={"schema":"nobody.ast.v1","ast":
                     {"kind":"module","line":1,"value":None,"children":[node]}}
            with self.subTest(value=val),self.assertRaises(ASTValidationError):from_json(payload)

    def test_depth_limit(self):
        node={"kind":"number","line":1,"value":"1","children":[]}
        for _ in range(140):node={"kind":"list","line":1,"value":None,"children":[node]}
        with self.assertRaises(ASTValidationError):
            from_json({"schema":"nobody.ast.v1","ast":node})

if __name__=="__main__":unittest.main()
