"""Nobody 0.18 conservative static typing, before execution."""
import unittest
from nobody.static_types import analyze

class StaticTypeTests(unittest.TestCase):
    def test_valid(self):
        self.assertTrue(analyze('nech n: cele = 2\nn = n + 3\nvypis(n)').ok)
    def test_initial_type_error(self):
        self.assertEqual(analyze('nech n: cele = "x"').diagnostics[0].code,'E_TYPE')
    def test_reassignment_type_error(self):
        self.assertEqual(analyze('nech n: cele = 1\nn = "x"').diagnostics[0].code,'E_TYPE')
    def test_bad_binary(self):
        self.assertEqual(analyze('nech x = "a" - 3').diagnostics[0].code,'E_BINARY')
    def test_undeclared_assignment(self):
        self.assertEqual(analyze('x = 1').diagnostics[0].code,'E_UNDECLARED')
    def test_scopes(self):
        self.assertTrue(analyze('nech x: cele = 1\nak pravda { nech x: text = "hi" }\nx = 4').ok)
    def test_captured_type_violation(self):
        self.assertFalse(analyze('nech x: cele = 1\nak pravda { x = "hi" }').ok)
    def test_ai_profile(self):
        self.assertFalse(analyze('ai_let x: ai_int = 1\nx = "hi"','ai').ok)
    def test_unknown_does_not_claim_safety(self):
        result=analyze('nech n: cele = mystery(2)')
        self.assertTrue(result.ok)
        self.assertTrue(result.warnings)
    def test_never_executes_code(self):
        self.assertTrue(analyze('pokial pravda { vypis("not executed") }').ok)

if __name__=='__main__':
    unittest.main()
