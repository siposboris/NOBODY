"""Nobody 0.13 catalogue, new scripts, and AST execution tests."""
import unittest
from nobody.catalog import languages, status
from nobody.languages import ALIASES
from nobody.translator import translate
from nobody.ast_vm import execute, ExecutionError
from nobody.bidi import direction_for_language

class NewMilestoneTests(unittest.TestCase):
    def test_249_entries_28_active(self):
        self.assertEqual(len(languages()),249)
        self.assertEqual(len(ALIASES),40)
        self.assertEqual(status()['planned_reference_variants'],210)

    def test_six_new_profiles_roundtrip(self):
        source='nech cislo = 3\nak cislo > 2 { vypis("Život") }'
        for lang in ('he','el','uk','fa','sw','nl'):
            with self.subTest(lang=lang):
                target=translate(source,'sk',lang)
                self.assertEqual(translate(target,lang,'sk'),source)
                output=[]
                execute(target,lang,output=lambda *args: output.append(args))
                self.assertEqual(output,[('Život',)])

    def test_rtl_hebrew_persian(self):
        for lang in ('he','fa'):
            self.assertEqual(direction_for_language(lang).direction,'rtl')

    def test_ast_function_and_branch(self):
        source='nech n = 2\nfunkcia dvoj(x) { vrat x * 2 }\nak n > 1 { vypis(dvoj(n) + 1) }'
        output=[]
        result=execute(source,output=lambda *args: output.append(args))
        self.assertEqual(output,[(5,)])
        self.assertGreater(result['steps'],0)

    def test_ast_loop_budget(self):
        output=[]
        execute('pre n v rozsah(3) { vypis(n) }',output=lambda *args:output.append(args))
        self.assertEqual(output,[(0,),(1,),(2,)])
        with self.assertRaisesRegex(ExecutionError,'step limit'):
            execute('pokial pravda { vypis(1) }',max_steps=15,output=lambda *args:None)

    def test_ast_typed_declarations(self):
        self.assertEqual(execute('nech n: cele = 1')['environment']['n'],1)
        with self.assertRaisesRegex(ExecutionError,'Type mismatch'):
            execute('nech n: cele = "bad"')

    def test_ast_unknown_call_rejected(self):
        with self.assertRaises(ExecutionError):
            execute('nech n = eval("1+1")')

if __name__=='__main__':
    unittest.main()
