#!/usr/bin/env python3
"""Flatten Substreams JSON output into one exact-schema JSON object per line.

Supports JSON objects with transfers, data.transfers, or data.output.transfers.
It does not silently claim to parse arbitrary terminal log formats.
"""
import argparse
import json
import sys

def locate_transfers(value):
    if isinstance(value, dict):
        if isinstance(value.get("transfers"), list):
            return value["transfers"]
        for key in ("data", "output", "value", "message"):
            if key in value:
                result = locate_transfers(value[key])
                if result is not None:
                    return result
    return None

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("input", help="JSON or JSONL input from a Substreams JSON output")
    parser.add_argument("output", help="Output JSONL path")
    args = parser.parse_args()
    count = 0
    with open(args.input, encoding="utf-8") as src, open(args.output, "w", encoding="utf-8") as dst:
        raw = src.read().strip()
        try:
            records = json.loads(raw)
            if not isinstance(records, list):
                records = [records]
        except json.JSONDecodeError:
            records = [json.loads(line) for line in raw.splitlines() if line.strip()]
        for record in records:
            transfers = locate_transfers(record)
            if transfers is None:
                continue
            for transfer in transfers:
                if not isinstance(transfer, dict):
                    raise ValueError("Transfer must be a JSON object")
                sender = transfer.get("from")
                recipient = transfer.get("to")
                amount = transfer.get("amount")
                if not isinstance(sender, str) or not isinstance(recipient, str):
                    raise ValueError("Transfer addresses must be strings")
                if isinstance(amount, str) and amount.isdecimal():
                    amount = int(amount)
                if type(amount) is not int or not (0 <= amount <= 2**64 - 1):
                    raise ValueError("Amount must be an unsigned 64-bit integer")
                dst.write(json.dumps({"from": sender, "to": recipient, "amount": amount}, separators=(",", ":")) + "\n")
                count += 1
    print(f"Exported {count} transfer records", file=sys.stderr)

if __name__ == "__main__":
    main()
