"""Nobody 0.17 lexical scope and closure tests."""
import unittest
from nobody.ast_vm import execute,ExecutionError
from nobody.translator import translate

class Scopes017(unittest.TestCase):
    def test_shadowing(self):
        out=[]
        execute('nech x = 1\nak pravda { nech x = 2\nvypis(x) }\nvypis(x)',
                output=lambda *a:out.append(a))
        self.assertEqual(out,[(2,),(1,)])

    def test_block_local_not_visible(self):
        with self.assertRaisesRegex(ExecutionError,'Unknown'):
            execute('ak pravda { nech secret = 5 }\nvypis(secret)',output=lambda *a:None)

    def test_closure_survives_outer_return(self):
        code='funkcia maker(x) { funkcia get() { vrat x }\nvrat get }\nnech f = maker(7)\nvypis(f())'
        output=[]
        execute(code,output=lambda *a:output.append(a))
        self.assertEqual(output,[(7,)])

    def test_closure_mutation(self):
        code='funkcia maker() { nech n: cele = 1\nfunkcia inc() { n = n + 1\nvrat n }\nvrat inc }\nnech f = maker()\nvypis(f())\nvypis(f())'
        output=[]
        execute(code,output=lambda *a:output.append(a))
        self.assertEqual(output,[(2,),(3,)])

    def test_typed_capture_rejects_invalid_reassignment(self):
        code='funkcia maker() { nech n: cele = 1\nfunkcia bad() { n = "wrong" }\nvrat bad }\nnech f = maker()\nf()'
        with self.assertRaisesRegex(ExecutionError,'Type mismatch'):
            execute(code)

    def test_recursion_and_guard(self):
        code='funkcia count(x) { ak x > 0 { vrat count(x - 1) }\nvrat 0 }\nvypis(count(5))'
        output=[]
        execute(code,output=lambda *a:output.append(a))
        self.assertEqual(output,[(0,)])
        with self.assertRaisesRegex(ExecutionError,'call depth'):
            execute(code,max_steps=100000,max_call_depth=3,output=lambda *a:None)

    def test_ai_syntax_closure(self):
        src='funkcia maker(x) { funkcia get() { vrat x }\nvrat get }\nnech f = maker(9)\nvypis(f())'
        output=[]
        execute(translate(src,'sk','ai'),'ai',output=lambda *a:output.append(a))
        self.assertEqual(output,[(9,)])

    def test_loop_variable_is_local(self):
        with self.assertRaisesRegex(ExecutionError,'Unknown'):
            execute('pre n v rozsah(2) { vypis(n) }\nvypis(n)',output=lambda *a:None)

if __name__=='__main__':unittest.main()
