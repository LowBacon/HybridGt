"""Summarize a VIctory diagnostics-build run log: menu vs world frame cost.

Usage: python -I tools/victory_logstats.py <run.log> [<run.log> ...]

Reads the per-second `victory_gpu::host: N fps, M ms/frame (...)` lines and
the `copystat: copies per frame` lines that follow them. World frames are
the lines from the first one under 10 fps on. Prints medians, because single
seconds vary a lot with the scene and with pipelines still being made.
"""

import re
import statistics
import sys

ANSI = re.compile(r"\x1b\[[0-9;]*m")
HOST = re.compile(
    r"victory_gpu::host: ([\d.]+) fps, ([\d.]+) ms/frame \(([\d.]+) preparing: copies ([\d.]+) ms ([\d.]+) MB, "
    r"textures ([\d.]+) ms ([\d.]+) MB; ([\d.]+) on the GPU; ([\d.]+) copying back ([\d.]+) MB\), ([\d.]+) draws/frame, "
    r"dropped (\d+) without a host pipeline \((\d+) being made\) and (\d+) indirect"
)
COPY = re.compile(r"copies per frame: ([\d.]+) submissions;.*?([\d.]+) page faults; ([\d.]+) watches ([\d.]+) ms, ([\d.]+) MB checked")
REUSED = re.compile(r"([\d.]+) watch askings reused")

FIELDS = ["fps", "ms/frame", "preparing", "copies ms", "copies MB", "textures ms", "textures MB", "GPU ms",
          "copy-back ms", "copy-back MB", "draws/frame", "dropped (no pipeline)", "being made", "dropped indirect"]
COPY_FIELDS = ["submissions", "page faults", "watches", "watch ms", "MB checked"]


def parse(path):
    rows, last = [], None
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            line = ANSI.sub("", line)
            m = HOST.search(line)
            if m:
                last = dict(zip(FIELDS, map(float, m.groups())))
                rows.append(last)
                continue
            m = COPY.search(line)
            if m and last is not None and "submissions" not in last:
                last.update(zip(COPY_FIELDS, map(float, m.groups())))
                r = REUSED.search(line)
                last["watch reused"] = float(r.group(1)) if r else 0.0
    return rows


def summary(rows):
    start = next((i for i, r in enumerate(rows) if r["fps"] < 10), None)
    menus, world = (rows, []) if start is None else (rows[:start], rows[start:])
    return menus, world


def median(rows, key):
    vals = [r[key] for r in rows if key in r]
    return statistics.median(vals) if vals else float("nan")


def main():
    keys = ["fps", "ms/frame", "preparing", "copies ms", "watch ms", "watches", "watch reused", "MB checked",
            "page faults", "GPU ms", "copy-back ms", "submissions", "draws/frame", "dropped (no pipeline)",
            "being made", "dropped indirect"]
    runs = [(p, *summary(parse(p))) for p in sys.argv[1:]]
    print(f"{'world-frame median':24}" + "".join(f"{p.split('/')[-1].split(chr(92))[-1]:>14}" for p, *_ in runs))
    print(f"{'seconds (menus/world)':24}" + "".join(f"{f'{len(m)}/{len(w)}':>14}" for _, m, w in runs))
    print(f"{'menu fps':24}" + "".join(f"{median(m, 'fps'):>14.1f}" for _, m, _ in runs))
    for k in keys:
        print(f"{k:24}" + "".join(f"{median(w, k):>14.1f}" for _, _, w in runs))


if __name__ == "__main__":
    main()
