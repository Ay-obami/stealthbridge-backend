#!/usr/bin/env python3
"""Compare the API's local read-only snapshot with canonical public Soroban data.

No generated claims or contract deployment; the GitHub API is a read-only
cross-repository check. CI fails if either repository gets out of sync.
"""
import json
from pathlib import Path
from urllib.request import urlopen
CANONICAL = "https://raw.githubusercontent.com/stealthbridge-labs/stealthbridge-contracts/main/deployments/testnet/manifest.json"
snapshot=json.loads(Path("deployments/testnet/manifest.json").read_text())
with urlopen(CANONICAL,timeout=15) as response:
    origin=json.load(response)
if snapshot!=origin:
    raise SystemExit("Contract manifest mismatch: sync from stealthbridge-contracts and verify before release")
if snapshot.get("status")!="not-deployed" or snapshot.get("verified") is not False:
    raise SystemExit("Unverified contract deployment must not be advertised by read-only backend")
print("Cross-repository contract deployment snapshot matches canonical un-deployed manifest.")
