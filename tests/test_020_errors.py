"""Stable numbered error diagnostics; existing symbolic IDs remain available."""
import unittest
from nobody.errors import CATALOG,BY_NUMBER,definition,format_diagnostic,render_exception
from nobody.static_types import analyze
from nobody.parser import parse_source,ParseError
from nobody.lexer import scan,LexError
from nobody.ast_vm import execute,ExecutionError

class NumberedErrors(unittest.TestCase):
    def test_number_uniqueness(self):
        self.assertEqual(len(CATALOG),len(BY_NUMBER))
        self.assertEqual(definition('E_TYPE').number,1001)
        self.assertEqual(definition('E_ARITY').number,1005)
    def test_catalog_has_actionable_descriptions(self):
        for item in CATALOG.values():
            with self.subTest(number=item.number):
                self.assertGreaterEqual(len(item.cause),28)
                self.assertGreaterEqual(len(item.fix),25)
    def test_type_number_and_specific_name(self):
        issue=analyze('nech vek: cele = "tridsat"').diagnostics[0]
        self.assertEqual((issue.code,issue.number),('E_TYPE',1001))
        self.assertIn('vek',issue.detailed_message)
        self.assertIn('Oprava:',issue.detailed_message)
    def test_wrong_number_of_arguments(self):
        issue=analyze('funkcia f(x) { vrat x }\nf(1,2)').diagnostics[0]
        self.assertEqual(issue.number,1005)
        self.assertIn('expects 1 arguments, got 2',issue.message)
    def test_numbered_warning(self):
        issue=analyze('nech x = mystery(1)').warnings[0]
        self.assertEqual(issue.number,1101)
    def test_lexer_classification(self):
        with self.assertRaises(LexError) as result:scan('vypis("never closed)')
        self.assertIn('N2001',render_exception(result.exception))
    def test_parser_classification(self):
        with self.assertRaises(ParseError) as result:parse_source('nech = 1')
        self.assertIn('N3003',render_exception(result.exception))
    def test_runtime_classification(self):
        with self.assertRaises(ExecutionError) as result:execute('vypis(missing)')
        self.assertIn('N4001',render_exception(result.exception))
    def test_lookup_coverage(self):
        self.assertIn('N1003',format_diagnostic('E_UNDECLARED','Cannot assign x',3))
        self.assertIn('riadok 3',format_diagnostic('E_UNDECLARED','Cannot assign x',3))
    def test_unknown_error_retains_specific_text(self):
        rendered=render_exception(ValueError('Custom precise problem with identifier foo'))
        self.assertIn('N9999',rendered)
        self.assertIn('identifier foo',rendered)

if __name__=='__main__':unittest.main()
