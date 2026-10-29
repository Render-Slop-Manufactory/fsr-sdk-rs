# SPDX-License-Identifier: MPL-2.0
"""Compatibility entry point for the recorded colour-signal test command."""
from fnmatch import fnmatchcase
import unittest


def load_tests(loader, tests, pattern):
    # Broad discovery already loads the canonical suite; avoid duplicate tests.
    if pattern and fnmatchcase('test_color_signal.py', pattern):
        return tests
    import test_color_signal
    return loader.loadTestsFromModule(test_color_signal)


if __name__ == '__main__':
    unittest.main()
