"""Nobody 0.21: conservative static inference of function return types."""
import unittest
from nobody.parser import parse_source
from nobody.return_types import infer_function_return, UNKNOWN
from nobody.static_types import analyze
from nobody.translator import translate

class ReturnTypeTests(unittest.TestCase):
    def infer(self,source):
        return infer_function_return(parse_source(source).children[0])
    def codes(self,source,language="sk"):
        return [d.code for d in analyze(source,language).diagnostics]

    def test_integer_function(self):
        self.assertEqual(self.infer('funkcia sedem() { vrat 7 }'),"int")
    def test_text_function(self):
        self.assertEqual(self.infer('funkcia pozdrav() { vrat "ahoj" }'),"str")
    def test_simple_local_expression(self):
        self.assertEqual(self.infer('funkcia pocet() { nech a = 2\nvrat a * 3 }'),"int")
    def test_uniform_branches(self):
        self.assertEqual(self.infer('funkcia volba(x) { ak x > 0 { vrat 1 } inak { vrat 2 } }'),"int")
    def test_divergent_branches_remain_unknown(self):
        self.assertEqual(self.infer('funkcia volba(x) { ak x > 0 { vrat 1 } inak { vrat "nie" } }'),UNKNOWN)
    def test_conditional_fallthrough_is_unknown(self):
        self.assertEqual(self.infer('funkcia volba(x) { ak x > 0 { vrat 1 } }'),UNKNOWN)
    def test_empty_function_returns_none(self):
        self.assertEqual(self.infer('funkcia prazdna() { }'),"none")
    def test_dynamic_call_remains_unknown(self):
        self.assertEqual(self.infer('funkcia volba() { vrat neznama() }'),UNKNOWN)
    def test_loops_are_not_speculatively_inferred(self):
        self.assertEqual(self.infer('funkcia volba() { pokial pravda { vrat 4 } }'),UNKNOWN)
    def test_known_return_mismatch_is_reported(self):
        source='funkcia vek() { vrat "tridsat" }\nnech n: cele = vek()'
        report=analyze(source)
        self.assertEqual(self.codes(source),["E_TYPE"])
        self.assertEqual(report.diagnostics[0].number,1001)
        self.assertIn("n",report.diagnostics[0].detailed_message)
    def test_known_return_correct_type_is_accepted(self):
        self.assertFalse(self.codes('funkcia vek() { vrat 30 }\nnech n: cele = vek()'))
    def test_ambiguous_return_not_falsely_rejected(self):
        self.assertFalse(self.codes('funkcia vek(x) { ak x > 0 { vrat 30 } inak { vrat "x" } }\nnech n: cele = vek(1)'))
    def test_function_arity_check_preserved(self):
        self.assertEqual(self.codes('funkcia vek() { vrat 30 }\nnech n = vek(1)'),["E_ARITY"])
    def test_machine_profile(self):
        source='funkcia vek() { vrat "tridsat" }\nnech n: cele = vek()'
        self.assertEqual(self.codes(translate(source,"sk","ai"),"ai"),["E_TYPE"])

if __name__=="__main__":
    unittest.main()
