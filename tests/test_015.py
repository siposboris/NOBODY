"""Nobody 0.15: five additional human scripts and deterministic AI profile."""
import json
import unittest
from nobody.languages import ALIASES, PREFERRED
from nobody.catalog import status
from nobody.translator import translate
from nobody.ast_vm import execute, ExecutionError
from nobody.parser import parse_source
from nobody.ai_ir import to_json

NEW=('ne','mr','gu','kn','te')
SOURCE='nech pocet = 3\nak pocet > 2 { vypis("Čau 世界") }\n'

class Milestone015(unittest.TestCase):
    def test_counts(self):
        self.assertEqual(len(ALIASES),40)
        self.assertEqual(status()['planned_reference_variants'],210)

    def test_native_roundtrips_and_execution(self):
        for language in NEW:
            with self.subTest(language=language):
                code=translate(SOURCE,'sk',language)
                self.assertEqual(translate(code,language,'sk'),SOURCE)
                outputs=[]
                execute(code,language,output=lambda *items:outputs.append(items))
                self.assertEqual(outputs,[("Čau 世界",)])
                self.assertTrue(any(ord(c)>127 for c in PREFERRED[language]['if']))

    def test_ai_profile_roundtrip(self):
        code=translate(SOURCE,'sk','ai')
        self.assertIn('ai_let pocet',code)
        self.assertIn('ai_if pocet',code)
        self.assertIn('ai_print(',code)
        self.assertEqual(translate(code,'ai','sk'),SOURCE)
        outputs=[]
        execute(code,'ai',output=lambda *a:outputs.append(a))
        self.assertEqual(outputs,[("Čau 世界",)])

    def test_ast_json_schema_is_deterministic(self):
        code=translate(SOURCE,'sk','ai')
        parsed=json.loads(to_json(code,'ai'))
        self.assertEqual(parsed['schema'],'nobody.ast.v1')
        self.assertEqual(parsed['ast']['children'][0]['kind'],'let')
        self.assertEqual(to_json(code,'ai'),to_json(code,'ai'))
        self.assertEqual([n.kind for n in parse_source(code,'ai').children],['let','if'])

    def test_annotated_assignment_guard(self):
        self.assertEqual(execute('nech n: cele = 1\nn = 2')['environment']['n'],2)
        with self.assertRaisesRegex(ExecutionError,'Type mismatch on reassignment'):
            execute('nech n: cele = 1\nn = "bad"')
        with self.assertRaisesRegex(ExecutionError,'Type mismatch on reassignment'):
            execute('nech n: logicke = pravda\nn = 1')

    def test_nested_assignment_guard(self):
        with self.assertRaisesRegex(ExecutionError,'Type mismatch on reassignment'):
            execute('nech n: cele = 1\nak pravda { n = "bad" }')

if __name__=='__main__':
    unittest.main()
