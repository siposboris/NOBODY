# Nobody 0.8 experimental prototype

**Date:** 2026-11-08

## Verified locally (outside GitHub CI)
- 49 Python unittest cases passed in the complete local Nobody 0.8 source tree.
- The previous 0.7 prototype ran 44 tests in the same environment.
- Source translation is still Nobody (.nbd) to Nobody (.nbd), not native compilation.

## Changes in 0.8
1. Lexical validation now rejects foreign syntax keywords even when they have fewer than three ASCII letters. Example: English `if` is rejected under Slovak syntax.
2. `translate_source()` validates the configured source language before translating, then validates the translated output language. The CLI already had input validation; this closes a library API gap.
3. Five regression tests added for mixed-language keywords, short keywords, uppercase syntax and Unicode strings.

## Limitations
- Some otherwise valid variable names collide with reserved words in other supported languages and are rejected.
- The prototype lexer is Python-tokenizer-based; a dedicated Nobody lexer/parser remains essential.
- These GitHub files currently document and test the new behavior. **The complete 0.7/0.8 Python bootstrap source has not yet been imported to this repository**, so these tests cannot currently run in GitHub CI.
- Not a self-hosted native compiler, memory sandbox, GPU compiler or complete language specification.

## Exact implementation delta for complete local prototype
```diff
--- a/nobody/compiler.py
+++ b/nobody/compiler.py
@@
-            if spelling not in current and len(spelling) >= 3:
+            if spelling not in current:
@@
 def translate_source(source: str, from_lang='sk', to_lang='en') -> str:
@@
+    check_single_language(source, from_lang)
     canonical = replace_words(source, ALIASES[from_lang])
-    return replace_words(canonical, PREFERRED[to_lang])
+    translated = replace_words(canonical, PREFERRED[to_lang])
+    check_single_language(translated, to_lang)
+    return translated
```

Do not tag this repository as v0.8 release until the complete code is uploaded and CI-tested.
