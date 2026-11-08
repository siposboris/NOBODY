"""Nobody v0.8 regression tests; run with the complete local prototype source."""
import unittest
from nobody.compiler import check_single_language, translate_source, NobodySyntaxError

class LanguageBarriers(unittest.TestCase):
    def test_short_english_if_rejected_in_slovak(self):
        with self.assertRaises(NobodySyntaxError):
            check_single_language('if pravda {\n    vypis(1)\n}\n', 'sk')

    def test_short_english_in_rejected_in_slovak(self):
        with self.assertRaises(NobodySyntaxError):
            check_single_language('nech in = 1\n', 'sk')

    def test_mixed_keywords_rejected_in_library_api(self):
        with self.assertRaises(NobodySyntaxError):
            translate_source('ak pravda {\n    print(1)\n}\n', 'sk', 'en')

    def test_keep_unicode_text(self):
        source = 'vypis("Život je krásny")\n'
        result = translate_source(source, 'sk','en')
        self.assertIn('Život je krásny', result)
        self.assertIn('print', result)

    def test_uppercase_keyword_rejected(self):
        with self.assertRaises(NobodySyntaxError):
            translate_source('Vypis(1)\n','sk','en')

if __name__ == '__main__':
    unittest.main()
