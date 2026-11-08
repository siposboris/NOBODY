"""Bidirectional layout must not change source logic."""
import unittest
from nobody.bidi import direction_for_language, check_bidi_controls, editor_advice
from nobody.compiler import translate_source
from nobody.lexer import scan
from nobody.parser import parse_source


class BidiTests(unittest.TestCase):
    def test_arabic_and_urdu_rtl_metadata(self):
        for language in ('ar', 'ur'):
            with self.subTest(language=language):
                self.assertEqual(direction_for_language(language).direction, 'rtl')
                self.assertTrue(direction_for_language(language).logical_order)

    def test_other_languages_ltr(self):
        for language in ('sk', 'en', 'ja', 'zh'):
            self.assertEqual(direction_for_language(language).direction, 'ltr')

    def test_unknown_language_rejected(self):
        with self.assertRaises(ValueError):
            direction_for_language('he')

    def test_no_reordering_round_trip(self):
        source = 'nech pocet = 3\nak pocet > 1 {\n vypis("Ahoj (3) مرحبا")\n}\n'
        for language in ('ar', 'ur'):
            translated = translate_source(source, 'sk', language)
            self.assertEqual(translate_source(translated, language, 'sk'), source)
            self.assertEqual(parse_source(translated, language).kind, 'module')

    def test_native_arabic_tokens_in_logical_order(self):
        names = [t.value for t in scan('إذا صحيح { اطبع("مرحبا") }') if t.kind == 'name']
        self.assertEqual(names, ['إذا', 'صحيح', 'اطبع'])

    def test_dangerous_bidi_controls_rejected_anywhere(self):
        for prefix in ('vypis("', '# ', 'nech cislo = '):
            for control in ('\u202e', '\u2066', '\u200f', '\u061c'):
                with self.subTest(prefix=prefix, control=hex(ord(control))):
                    with self.assertRaisesRegex(ValueError, 'Forbidden bidirectional control'):
                        scan(prefix + control + 'abc")')

    def test_arabic_diacritics_not_bidi_controls(self):
        check_bidi_controls('إذا صحيح { اطبع("السَّلَامُ") }')
        self.assertIn('UTF-8', editor_advice('ar'))


if __name__ == '__main__':
    unittest.main()
