#!/usr/bin/env python3
"""Verify unchanged public APIs at a Git revision in an isolated archive."""
import argparse
import hashlib
import json
import os
import platform
from pathlib import Path
import shutil
import signal
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[1]
p = argparse.ArgumentParser(description=__doc__)
p.add_argument("--revision", default="HEAD")
p.add_argument("--harness", action="append", required=True)
p.add_argument("--timeout", type=int, default=180, help="Seconds per harness; timeout is inconclusive")
p.add_argument("--output", type=Path, required=True)
a = p.parse_args()
sha = subprocess.check_output(["git", "rev-parse", a.revision + "^{commit}"], cwd=ROOT, text=True).strip()
if a.timeout <= 0:
    p.error("--timeout must be positive")
if a.output.exists() and any(a.output.iterdir()):
    p.error("--output must be empty or new (preserve earlier evidence)")
a.output.mkdir(parents=True, exist_ok=True)
a.output = a.output.resolve()
work = Path(tempfile.mkdtemp(prefix="rusty-money-kani-"))
repo = work / "repository"
repo.mkdir()
archive = subprocess.check_output(["git", "archive", sha], cwd=ROOT)
subprocess.run(["tar", "-x", "-C", str(repo)], input=archive, check=True)
proofs = work / "verification"
shutil.copytree(ROOT / "verification" / "src", proofs / "src")
manifest = (ROOT / "verification" / "Cargo.toml").read_text().replace('path = ".."', 'path = "../repository"')
(proofs / "Cargo.toml").write_text(manifest)
lock = ROOT / "verification" / "Cargo.lock"
if lock.exists():
    shutil.copyfile(lock, proofs / "Cargo.lock")
env = dict(os.environ, CARGO_HUSKY_DONT_INSTALL_HOOKS="true", CARGO_NET_OFFLINE="true")
subprocess.run(["cargo", "metadata", "--locked", "--offline", "--format-version", "1"], cwd=proofs, env=env, stdout=subprocess.DEVNULL, check=True)
version = subprocess.check_output(["cargo", "kani", "--version"], env=env, text=True).strip()
report = {"revision": sha, "kani": version, "platform": platform.platform(), "workdir": str(work),
          "harness_sha256": hashlib.sha256((proofs / "src/lib.rs").read_bytes()).hexdigest(),
          "timeout_seconds": a.timeout, "results": []}
shutil.copyfile(proofs / "src/lib.rs", a.output / "harnesses.rs")
shutil.copyfile(proofs / "Cargo.toml", a.output / "Cargo.toml")
(a.output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
for harness in a.harness:
    command = ["cargo", "kani", "--harness", "proofs::" + harness, "--exact", "--output-format", "terse",
               "-Z", "concrete-playback", "--concrete-playback=print"]
    start = time.monotonic()
    log = a.output / (harness + ".log")
    with log.open("w") as output:
        proc = subprocess.Popen(command, cwd=proofs, env=env, stdout=output,
                                stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code = proc.wait(timeout=a.timeout)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGTERM)
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(proc.pid, signal.SIGKILL)
                proc.wait()
            code = None
    content = log.read_text()
    if lock.exists() and (proofs / "Cargo.lock").read_bytes() != lock.read_bytes():
        raise RuntimeError("Kani changed the dependency lock; results require investigation")
    if code is None:
        status = "TIMEOUT"
    elif code == 0 and "VERIFICATION:- SUCCESSFUL" in content:
        status = "PASS"
    elif "unwinding assertion loop" in content:
        status = "INSUFFICIENT_UNWIND"
    elif "VERIFICATION:- FAILED" in content:
        status = "FAIL"
    else:
        status = "TOOL_ERROR"
    result = {"harness": harness, "status": status, "exit_code": code,
              "seconds": round(time.monotonic() - start, 2), "command": command}
    report["results"].append(result)
    (a.output / "summary.json").write_text(json.dumps(report, indent=2) + "\n")
    print(f"{sha[:8]} {harness}: {status} ({result['seconds']}s)", flush=True)
# Retain source, dependency lock, and logs for inspection, including inconclusive runs.
if (proofs / "Cargo.lock").exists():
    shutil.copyfile(proofs / "Cargo.lock", a.output / "Cargo.lock")
raise SystemExit(0 if all(r["status"] == "PASS" for r in report["results"]) else 1)
