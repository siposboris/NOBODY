"""Regression tests: path-sensitive static joins and known function arity."""
import unittest
from nobody.static_types import analyze
from nobody.translator import translate

class BranchAndFunctionTyping(unittest.TestCase):
    def test_known_function_wrong_arity(self):
        report=analyze('funkcia double(x) { vrat x + x }\nvypis(double(1,2))')
        self.assertIn('E_ARITY',[d.code for d in report.diagnostics])
    def test_known_function_arity_ok(self):
        self.assertTrue(analyze('funkcia double(x) { vrat x + x }\nvypis(double(2))').ok)
    def test_nested_function_wrong_arity(self):
        report=analyze('funkcia outer() { funkcia inner(x) { vrat x }\nvrat inner(1,2) }')
        self.assertIn('E_ARITY',[d.code for d in report.diagnostics])
    def test_shadowed_function_arity(self):
        report=analyze('funkcia f(x) { vrat x }\nak pravda { funkcia f() { vrat 1 }\nf(1) }\nf(2)')
        self.assertEqual([d.code for d in report.diagnostics],['E_ARITY'])
    def test_if_without_else_keeps_unknown_on_changed_type(self):
        report=analyze('nech x = 1\nak nepravda { x = "str" }\nnech vysledok = x - 2')
        self.assertTrue(report.ok,report.diagnostics)
    def test_if_else_both_paths_same_type(self):
        report=analyze('nech x = 1\nak pravda { x = "str" } inak { x = "text" }\nnech vysledok = x - 2')
        self.assertIn('E_BINARY',[d.code for d in report.diagnostics])
    def test_if_else_split_types_unknown(self):
        report=analyze('nech x = 1\nak pravda { x = "str" } inak { x = 3 }\nnech vysledok = x - 2')
        self.assertTrue(report.ok,report.diagnostics)
    def test_while_may_run_zero_times(self):
        report=analyze('nech x = 1\npokial nepravda { x = "str" }\nnech vysledok = x - 2')
        self.assertTrue(report.ok,report.diagnostics)
    def test_known_type_error_in_branch_still_reported(self):
        report=analyze('nech x: cele = 1\nak nepravda { x = "str" }')
        self.assertIn('E_TYPE',[d.code for d in report.diagnostics])
    def test_ai_profile_has_same_rules(self):
        src='funkcia square(x) { vrat x * x }\nvypis(square(1,2))'
        report=analyze(translate(src,'sk','ai'),'ai')
        self.assertIn('E_ARITY',[d.code for d in report.diagnostics])

if __name__=='__main__':unittest.main()
