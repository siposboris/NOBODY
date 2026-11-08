"""Regression checks of all 40 supported syntax dictionaries."""
import unittest
from nobody.languages import ALIASES, NATIVE_SCRIPT_PROFILES
from nobody.translator import translate, validate_language, SyntaxLanguageError


class NativeScriptTranslationTests(unittest.TestCase):
    SOURCE = 'nech cislo = 3\nak cislo > 1 {\n vypis("Čau 世界 if 如果") # Čau print\n} inak {\n vypis("Nie")\n}\n'

    def test_all_40(self):
        self.assertEqual(len(ALIASES), 40)
        for lang in ALIASES:
            with self.subTest(lang=lang):
                dest = translate(self.SOURCE, 'sk', lang)
                self.assertEqual(translate(dest, lang, 'sk'), self.SOURCE)
                self.assertIn('"Čau 世界 if 如果"', dest)
                self.assertIn('# Čau print', dest)

    def test_native_written_keywords(self):
        for code in ('ru','zh','hi','ar','bn','ur','ja','ko','ta','th'):
            with self.subTest(code=code):
                self.assertTrue(any(ord(c) > 127 for c in next(iter(ALIASES[code]))))
                self.assertIn(code, NATIVE_SCRIPT_PROFILES)

    def test_reject_mixed_syntax(self):
        with self.assertRaises(SyntaxLanguageError):
            translate('ak pravda { print(3) }', 'sk', 'ja')

    def test_reject_uppercase(self):
        with self.assertRaises(SyntaxLanguageError):
            validate_language('Vypis("Ahoj")', 'sk')

    def test_preserve_expressions(self):
        self.assertIn('如果', translate('ak pravda { vypis("ahoj") }', 'sk', 'zh'))
        self.assertIn('если', translate('ak pravda { vypis("ahoj") }', 'sk', 'ru'))
        self.assertIn('إذا', translate('ak pravda { vypis("ahoj") }', 'sk', 'ar'))


if __name__ == '__main__':
    unittest.main()
