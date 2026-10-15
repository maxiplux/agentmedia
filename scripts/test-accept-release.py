#!/usr/bin/env python3
"""Offline regression: public acceptance must expose updater diagnostics and fail."""
import json
import pathlib
import runpy
import subprocess
import unittest
from unittest.mock import patch


class Diagnostics(unittest.TestCase):
    def setUp(self):
        # Importing the harness must not perform downloads or installation.
        with patch('urllib.request.urlopen', side_effect=AssertionError('unexpected network')):
            self.cli = runpy.run_path(str(pathlib.Path(__file__).with_name('accept-release.py')))['run_cli']

    def test_exit_failure_preserves_structured_error_and_stderr(self):
        diagnostic = json.dumps({'ok': False, 'errors': [{'code': 'installation', 'message': 'GitHub Releases returned HTTP 403; use the Releases page or try later'}]})
        result = subprocess.CompletedProcess([], 3, diagnostic, 'fixture stderr')
        with patch('subprocess.run', return_value=result) as run:
            with self.assertRaisesRegex(RuntimeError, 'exit 3') as caught:
                self.cli('fixture-binary', ['update', '--check'], {}, pathlib.Path('.'))
        self.assertIn(diagnostic, str(caught.exception))
        self.assertIn('fixture stderr', str(caught.exception))
        self.assertEqual(run.call_count, 1)

    def test_timeout_is_not_retried(self):
        with patch('subprocess.run', side_effect=subprocess.TimeoutExpired([], 300)) as run:
            with self.assertRaisesRegex(RuntimeError, 'no automatic retry'):
                self.cli('fixture-binary', ['update', '--force'], {}, pathlib.Path('.'))
        self.assertEqual(run.call_count, 1)

    def test_structured_failure_with_zero_exit_is_still_rejected(self):
        result = subprocess.CompletedProcess([], 0, '{"ok": false, "errors": ["fixture failure"]}', '')
        with patch('subprocess.run', return_value=result):
            with self.assertRaisesRegex(RuntimeError, 'fixture failure'):
                self.cli('fixture-binary', ['update', '--check'], {}, pathlib.Path('.'))


if __name__ == '__main__':
    unittest.main()
