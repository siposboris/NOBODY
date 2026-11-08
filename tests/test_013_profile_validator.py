import unittest
from nobody.profile_validator import audit_profiles, profile_summary

class ProfileValidationTests(unittest.TestCase):
    def test_no_lexical_profile_errors(self):
        self.assertFalse([issue for issue in audit_profiles() if issue.severity=='error'])
    def test_count(self):
        self.assertEqual(profile_summary()['profiles'],34)

if __name__=='__main__':unittest.main()
