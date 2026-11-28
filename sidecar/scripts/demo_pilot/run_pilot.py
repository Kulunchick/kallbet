#!/usr/bin/env python3
"""Demo pilot driver: one demo at a time -> download (collector) -> extract (bsdtar) -> parse (demoparser2, python -I) -> delete.

Needs the release collector built with --features hltv-impersonate. Stops at the first sign of a block (stop.json or a
STOPPED note in the collector output) and never retries. Keeps only the per-round parquet tables and pilot_log.tsv.

  python run_pilot.py --selection .../derived/demo-pilot/selection.tsv --tmp D:/Projects/KallBet/target/demo-pilot-tmp \
      --out .../derived/demo-pilot --collector kallbet-ai.exe --python <venv python> [--max N]
"""
import argparse, csv, os, shutil, subprocess, sys, time

HERE = os.path.dirname(os.path.abspath(__file__))
BSDTAR = "C:/Windows/System32/tar.exe"


def du(path):
    tot = 0
    for r, _, fs in os.walk(path):
        for f in fs:
            try:
                tot += os.path.getsize(os.path.join(r, f))
            except OSError:
                pass
    return tot


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--selection", required=True)
    ap.add_argument("--tmp", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--collector", required=True)
    ap.add_argument("--python", required=True, help="interpreter with demoparser2 and pandas/pyarrow")
    ap.add_argument("--max", type=int, default=0)
    ap.add_argument("--ios-version", default="17.2.1")
    a = ap.parse_args()
    rows = list(csv.DictReader(open(a.selection, encoding="utf8"), delimiter="\t"))
    if a.max:
        rows = rows[: a.max]
    os.makedirs(a.out, exist_ok=True)
    log_path = os.path.join(a.out, "pilot_log.tsv")
    cols = ["match_id", "demo_id", "event", "teams", "rar_bytes", "download_s", "extract_s", "dem_files", "dem_bytes",
            "parse_s", "rounds", "peak_tmp_bytes", "status"]
    done = set()
    if os.path.exists(log_path):
        done = {r["demo_id"] for r in csv.DictReader(open(log_path, encoding="utf8"), delimiter="\t") if r["status"] == "ok"}
    new_log = not os.path.exists(log_path)
    logf = open(log_path, "a", encoding="utf8", newline="")
    lw = csv.writer(logf, delimiter="\t", lineterminator="\n")
    if new_log:
        lw.writerow(cols)
    for r in rows:
        demo_id = r["demo_id"]
        if demo_id in done:
            continue
        if os.path.exists(os.path.join(a.tmp, "stop.json")):
            print("STOP: stop.json present, not requesting anything", file=sys.stderr)
            break
        idf = os.path.join(a.tmp, f"id-{demo_id}.tsv")
        open(idf, "w").write(demo_id + "\n")
        rar = os.path.join(a.tmp, "demos", f"{demo_id}.bin")
        t0 = time.time()
        if not os.path.exists(rar):
            p = subprocess.run([a.collector, "hltv-collect", "--demo-ids", idf, "--out", a.tmp, "--min-interval-secs", "6",
                                "--ios-version", a.ios_version, "--max-requests", "4"], capture_output=True, text=True)
            msg = (p.stdout + p.stderr).strip().splitlines()[-1:] or [""]
            print(msg[0][:300], flush=True)
            if "STOPPED" in p.stdout or p.returncode != 0 or os.path.exists(os.path.join(a.tmp, "stop.json")):
                lw.writerow([r["match_id"], demo_id, r["event_name"], r["teams"], 0, round(time.time() - t0, 1), "", "", "", "", "", "", "stopped/failed: " + msg[0][:120]])
                logf.flush()
                break
        download_s = time.time() - t0
        if download_s < 1:  # fetched by an earlier run: take its recorded time
            for row in csv.DictReader(open(os.path.join(a.tmp, "demos.tsv"), encoding="utf8"), delimiter="	"):
                if row["demo_id"] == demo_id:
                    download_s = float(row["seconds"])
        rar_bytes = os.path.getsize(rar)
        ex = os.path.join(a.tmp, f"x{demo_id}")
        shutil.rmtree(ex, ignore_errors=True)
        os.makedirs(ex)
        t1 = time.time()
        p = subprocess.run([BSDTAR, "-xf", rar, "-C", ex], capture_output=True, text=True)
        extract_s = time.time() - t1
        dems = sorted(f for f in os.listdir(ex) if f.endswith(".dem"))
        peak = du(a.tmp)
        status, nrounds, parse_s = "ok", 0, 0.0
        if p.returncode != 0 or not dems:
            status = "extract failed: " + (p.stderr or "no .dem")[:100]
        for f in dems:
            t2 = time.time()
            dem_name = os.path.splitext(f)[0]
            outp = os.path.join(a.out, f"rounds-{r['match_id']}-{dem_name[-24:]}.parquet".replace(" ", "_"))
            q = subprocess.run([a.python, "-I", os.path.join(HERE, "parse_demo.py"), os.path.join(ex, f), outp,
                                "--match-id", r["match_id"], "--demo-id", demo_id], capture_output=True, text=True)
            parse_s += time.time() - t2
            if q.returncode != 0:
                status = "parse failed: " + q.stderr.strip().splitlines()[-1][:100]
            else:
                nrounds += int(q.stdout.split("rounds")[1].split()[0])
        dem_bytes = sum(os.path.getsize(os.path.join(ex, f)) for f in dems)
        shutil.rmtree(ex, ignore_errors=True)
        if status == "ok":
            os.remove(rar)  # parsed table kept, source deleted
        lw.writerow([r["match_id"], demo_id, r["event_name"], r["teams"], rar_bytes, round(download_s, 1), round(extract_s, 1),
                     len(dems), dem_bytes, round(parse_s, 1), nrounds, peak, status])
        logf.flush()
        print(r["match_id"], r["teams"], "rar", rar_bytes, "dems", len(dems), "rounds", nrounds, status, flush=True)
    logf.close()


if __name__ == "__main__":
    main()
