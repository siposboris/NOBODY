"""Tests for independent Nobody lexical scanner."""
import unittest
from nobody.lexer import scan, LexError


class NobodyLexerTests(unittest.TestCase):
    def test_identifiers_with_positions(self):
        names = [t for t in scan('nech cislo = 42\nvypis(cislo)\n') if t.kind == 'name']
        self.assertEqual([(t.value, t.line, t.column) for t in names],
                         [('nech', 1, 1), ('cislo', 1, 6), ('vypis', 2, 1), ('cislo', 2, 7)])

    def test_unicode_in_strings_and_comments(self):
        ts = scan('vypis("Čau IF PRINT") # Živá poznámka\n')
        self.assertEqual([t.kind for t in ts if t.kind in ('string', 'comment')],
                         ['string', 'comment'])

    def test_triple_strings(self):
        ts = scan('vypis("""if\nprint\n""")')
        self.assertEqual(sum(t.kind == 'string' for t in ts), 1)

    def test_reject_bad_identifiers(self):
        for source in ['Vypis(1)', 'vypis(1) × 2']:
            with self.subTest(source=source), self.assertRaises(LexError):
                scan(source)

    def test_unterminated_string(self):
        with self.assertRaisesRegex(LexError, 'Unterminated string'):
            scan('vypis("ahoj)')

    def test_comment_does_not_tokenize_words(self):
        ts = scan('# if IF ČAU\nnech a = 1')
        self.assertEqual([t.value for t in ts if t.kind == 'name'], ['nech','a'])


if __name__ == '__main__':
    unittest.main()
