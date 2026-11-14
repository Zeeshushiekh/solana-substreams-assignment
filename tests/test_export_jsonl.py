import importlib.util
import json
import pathlib
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("export_jsonl", ROOT / "scripts" / "export_jsonl.py")
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

class ExportTests(unittest.TestCase):
    def test_nested_envelopes(self):
        self.assertEqual(mod.locate_transfers({"data": {"output": {"transfers": [{"from": "a", "to": "b", "amount": "42"}]}}})[0]["amount"], "42")
    def test_empty_block(self):
        self.assertEqual(mod.locate_transfers({"transfers": []}), [])
    def test_unrelated_record(self):
        self.assertIsNone(mod.locate_transfers({"log": "no transfers"}))

if __name__ == "__main__":
    unittest.main()
