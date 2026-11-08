"""Nobody 0.14: six additional syntax profiles, mutable assignment in the AST."""
import unittest
from nobody.languages import ALIASES, PREFERRED
from nobody.catalog import status
from nobody.translator import translate
from nobody.ast_vm import execute, ExecutionError
from nobody.parser import parse_source
from nobody.bidi import direction_for_language

NEW = ('bg','ro','sv','fi','hy','ka')
SOURCE = 'nech pocet = 3\nak pocet > 2 { vypis("Život 世界") }\n'

class Milestone014(unittest.TestCase):
    def test_profile_counts(self):
        self.assertEqual(len(ALIASES),40)
        self.assertEqual(status()['planned_reference_variants'],210)

    def test_roundtrips(self):
        for code in NEW:
            with self.subTest(code=code):
                target=translate(SOURCE,'sk',code)
                self.assertEqual(translate(target,code,'sk'),SOURCE)
                self.assertEqual(parse_source(target,code).kind,'module')

    def test_original_scripts(self):
        for code in ('bg','hy','ka'):
            self.assertTrue(any(ord(ch)>127 for ch in PREFERRED[code]['if']))
            self.assertEqual(direction_for_language(code).direction,'ltr')

    def test_all_six_execute(self):
        for code in NEW:
            with self.subTest(code=code):
                output=[]
                execute(translate(SOURCE,'sk',code),code,output=lambda *a:output.append(a))
                self.assertEqual(output,[("Život 世界",)])

    def test_assignment_inside_loop(self):
        src='nech count = 0\npokial count < 3 { count = count + 1 }\nvypis(count)'
        out=[]
        result=execute(src,output=lambda *a:out.append(a))
        self.assertEqual(out,[(3,)])
        self.assertEqual(result['environment']['count'],3)

    def test_undeclared_assignment(self):
        with self.assertRaisesRegex(ExecutionError,'undeclared'):
            execute('count = 1')

    def test_nonvariable_assignment(self):
        with self.assertRaises(Exception):
            execute('vypis(1) = 2')

if __name__=='__main__':
    unittest.main()
