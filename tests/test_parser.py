"""Regression tests for experimental Nobody AST parser."""
import unittest
from nobody.parser import parse_source, ParseError


class ParserTests(unittest.TestCase):
    def test_slovak_function_condition(self):
        src = '''nech cislo = 10
funkcia dvoj(x) { vrat x * 2 }
ak cislo > 5 { vypis(dvoj(cislo)) } inak { vypis(0) }
'''
        tree = parse_source(src, 'sk')
        self.assertEqual([n.kind for n in tree.children], ['let', 'function', 'if'])
        self.assertEqual(tree.children[1].value, ('dvoj', ('x',)))
        self.assertEqual(len(tree.children[2].children), 2)

    def test_english_and_czech(self):
        self.assertEqual(parse_source('let n = 1\nif n < 2 { print(n) }','en').children[1].kind, 'if')
        self.assertEqual(parse_source('necht n = 1\npokud pravda { vypis(n) }','cs').children[1].kind, 'if')

    def test_for_while(self):
        program = 'pre n v rozsah(4) { vypis(n) }\npokial nepravda { prerus }'
        self.assertEqual([n.kind for n in parse_source(program).children], ['for', 'while'])

    def test_precedence(self):
        value = parse_source('nech x = 1 + 2 * 3').children[0].children[0]
        self.assertEqual((value.kind, value.value, value.children[1].value), ('binary','+','*'))

    def test_unicode_string(self):
        tree = parse_source('nech x = [1,2]\nvypis("Čau IF") # IF\n')
        self.assertEqual(tree.children[0].children[0].kind, 'list')

    def test_no_foreign_syntax(self):
        with self.assertRaises(ParseError):
            parse_source('if pravda { vypis(1) }', 'sk')

    def test_no_uppercase_syntax(self):
        with self.assertRaises(ValueError):
            parse_source('Vypis(1)')

    def test_malformed_braces(self):
        for source in ['ak pravda { vypis(1)', 'nech x = (1 + 2', '}']:
            with self.subTest(source=source), self.assertRaises(ParseError):
                parse_source(source)

    def test_unknown_declaration(self):
        with self.assertRaises(ParseError):
            parse_source('nech = 2')

    def test_typed_declarations_explicitly_unsupported(self):
        self.assertEqual(parse_source('nech n: cele = 1').children[0].kind, 'typed_let')


if __name__ == '__main__':
    unittest.main()
