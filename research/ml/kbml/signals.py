"""`uv run python -m kbml.signals`: phase 5 (SIGNALS.md). Primary first: `lgbm-ext` + A on the
priced scored Matches; then everything exploratory. Per-Match predictions of every configuration
go to the cache (`signals/preds.parquet`, for `kbml.spa`), the summary to `signals/results.json`.
"""
from __future__ import annotations

import argparse
import glob
import gzip
import json
import re
import sys
import time
from pathlib import Path

import numpy as np
import pandas as pd

from . import evaluate, features, fusion, stats
from . import features_signals as fs
from .forecaster import EXT
from .market_detail import load_detail, logit
from .run import with_hltv
from .signal_eval import compare
from .table import cache_dir, load_table

BASES = {"lgbm-ext": ("lgbm", "p3_all"), "logreg-ext": ("logreg", "p3_rw")}
NEW_GROUPS = ["veto_eb", "openskill", "travel"]
VETO_VARIANTS = {"veto_eb_noreset": {"reset": False}, "veto_eb_noshrink": {"shrink": False}}


def travel_inputs(archive: Path):
    """{stage_id: (country, offline)} and {team_id: country} from PandaScore rows; a Stage without a
    country takes its serie's Liquipedia infobox `|country=` (latest saved revision)."""
    stages, homes, series_of = {}, {}, {}
    for f in sorted(glob.glob(str(archive / "panda-*.json.gz"))):
        rows = json.loads(gzip.decompress(Path(f).read_bytes()))
        if not rows:
            continue
        t = rows[0].get("tournament") or {}
        typ = t.get("type")
        stages[t.get("id")] = (t.get("country") or None, None if typ is None else typ == "offline")
        series_of[t.get("id")] = rows[0].get("serie_id")
        for r in rows:
            for o in r.get("opponents") or []:
                op = o.get("opponent") or {}
                if isinstance(op.get("id"), int) and op.get("location"):
                    homes[op["id"]] = op["location"]
    lp = {}
    for f in sorted(glob.glob(str(archive / "roster-*.json.gz"))):
        head = Path(f).name[len("roster-"):].split("-")[0]
        if not head.isdigit():
            continue
        page = json.loads(gzip.decompress(Path(f).read_bytes()))["query"]["pages"][0]
        revs = page.get("revisions") or []
        if not revs:
            continue
        rev = revs[0]
        text = rev.get("content") or (rev.get("slots") or {}).get("main", {}).get("content", "")
        c = re.search(r"^\|\s*country\s*=\s*([^\n|]+)", text, re.M)
        ty = re.search(r"^\|\s*type\s*=\s*(\w+)", text, re.M)
        if c:
            code = fs.COUNTRY_NAMES.get(c.group(1).strip().lower())
            lp[int(head)] = (code, None if not ty else ty.group(1).lower() == "offline")
    filled = 0
    for sid, (country, offline) in list(stages.items()):
        if country is None and series_of.get(sid) in lp:
            c2, off2 = lp[series_of[sid]]
            stages[sid] = (c2, offline if offline is not None else off2)
            filled += c2 is not None
    return stages, homes, filled


def _features(m, maps, hltv, cache: Path, log):
    f = cache / "signals" / "X_new.parquet"
    X3 = pd.read_parquet(cache / "spa" / "p3_all_X.parquet")
    groups3 = features.resolve("p3_all")
    kinds = {c: k for g in groups3 for c, k in features.REGISTRY[g].columns.items()}
    for g in NEW_GROUPS:
        kinds.update(features.REGISTRY[g].columns)
    for name in VETO_VARIANTS:
        kinds.update({f"{c}__{name}": k for c, k in fs.VetoEB.columns.items()})
    if f.exists():
        Xn = pd.read_parquet(f)
    else:
        t = time.time()
        st, homes, filled = travel_inputs(EXT)
        fs.Travel.stages, fs.Travel.homes = st, homes
        log(f"travel: {sum(c is not None for c, _ in st.values())}/{len(st)} Stages with a country "
            f"({filled} from Liquipedia), {len(homes)} team locations")
        Xn, _ = features.compute(m, maps, NEW_GROUPS, hltv)
        Xn = _series(Xn, fs.VetoEB.last.pending, log)
        for name, flags in VETO_VARIANTS.items():
            for k, v in flags.items():
                setattr(fs.VetoEB, k, v)
            Xv, _ = features.compute(m, maps, ["veto_eb"], hltv)
            for k in flags:
                setattr(fs.VetoEB, k, not flags[k])
            Xv = _series(Xv, fs.VetoEB.last.pending, log)
            Xn = Xn.join(Xv.add_suffix(f"__{name}"))
        Xn.to_parquet(f)
        log(f"new features {Xn.shape} [{time.time() - t:.0f}s]")
    X = X3.join(Xn)
    return X, kinds


TIMINGS: dict = {}


def _device():
    try:
        import torch
        return "cuda" if torch.cuda.is_available() else "numpy"
    except ImportError:
        return "numpy"


def _series(X, pending, log):
    """Exact veto integration for every pending Match, on the GPU when there is one (retrying with
    smaller chunks on CUDA out-of-memory), else numpy."""
    dev = _device()
    t = time.time()
    chunk = 512
    while True:
        try:
            ids = list(pending)
            p = fs.veto_series(np.stack([pending[i] for i in ids]), dev, chunk) if ids else np.array([])
            break
        except RuntimeError as e:  # torch.cuda.OutOfMemoryError is a RuntimeError
            if "out of memory" not in str(e).lower() or chunk <= 16:
                raise
            import torch
            torch.cuda.empty_cache()
            chunk //= 2
            time.sleep(10)
    p = np.clip(p, 1e-4, 1 - 1e-4)
    X.loc[ids, "veb_series_logit"] = np.log(p / (1 - p))
    TIMINGS.setdefault("veto_series", []).append({"device": dev, "matches": len(ids), "seconds": round(time.time() - t, 2)})
    log(f"veto integration: {len(ids)} Matches on {dev} [{time.time() - t:.1f}s]")
    return X


def _cols(fset, kinds, add=(), drop=()):
    cols = [c for g in features.SETS[fset] if g not in drop for c in features.REGISTRY[g].columns]
    cols += [c for g in add for c in features.REGISTRY[g].columns]
    return {c: kinds[c] for c in cols}


def _summ(name, p_var, p_base, pred, mask=None):
    sel = np.ones(len(pred), bool) if mask is None else mask
    y = pred["team_a_won"].to_numpy()[sel]
    ev = pred["event_id"].to_numpy()[sel]
    return {"name": name, **compare(np.asarray(p_var)[sel], np.asarray(p_base)[sel], y, ev)}


def pool_changes(m, maps, hltv):
    """The pool changes the point-in-time ActivePool detects over the whole archive."""
    g = fs.VetoEB()
    from .history import History, Timeline, match_infos
    tl = Timeline(History(m, maps, hltv[1]))
    infos = sorted(match_infos(m), key=lambda i: (i.cutoff, i.row["match_id"]))
    for info in infos:
        new = tl.advance_to(info.cutoff)
        g.update(new)
        if info.cutoff - g.now > 7 * fs.DAY:
            g.pool.refresh(info.cutoff)
            g.now = info.cutoff
    return [(pd.Timestamp(t, unit="us", tz="UTC").strftime("%Y-%m-%d"), a, d) for t, a, d in g.pool.changes]


def run(log=print) -> dict:
    t0 = time.time()
    cache = cache_dir(EXT)
    (cache / "signals").mkdir(parents=True, exist_ok=True)
    m, maps, _ = load_table(EXT, log=log)
    m, hltv = with_hltv(m, EXT)
    X, kinds = _features(m, maps, hltv, cache, log)
    detail = load_detail(EXT, cache, log)
    log(f"inputs ready [{time.time() - t0:.0f}s]")
    res = {"primary": None, "rows": [], "notes": {}}
    preds = {}

    base_pred = {}
    for name, (model, fset) in BASES.items():
        pred = evaluate.walk_forward(m, X, _cols(fset, kinds), model, train_blocks=fusion.TRAIN_BLOCKS,
                                     log=lambda *_: None)
        base_pred[name] = pred
        preds[name] = pred["variant"].to_numpy()
        log(f"{name}: log-loss {stats.log_loss(pred.variant.to_numpy(), pred.team_a_won.to_numpy()).mean():.4f}")
    ref = base_pred["lgbm-ext"]
    pos = pd.Series(np.arange(len(m)), index=m["match_id"].to_numpy())
    pos = pos[~pos.index.duplicated()]
    rows = pos.loc[ref["match_id"]].to_numpy()  # row positions in `m` of the scored Matches
    mk = ref["has_market"].to_numpy()

    # ---------------- A: two-stage fusion (primary = lgbm-ext, core, priced Matches)
    s1 = {}
    for name, (model, fset) in BASES.items():
        t = time.time()
        s1[name] = fusion.stage1(m, X, fusion.fundamental_columns(fset, kinds), model, log=log)
        log(f"stage 1 {name} [{time.time() - t:.0f}s]")
    pd.DataFrame({k: v.to_numpy() for k, v in s1.items()}, index=m["match_id"]).to_parquet(cache / "signals" / "stage1.parquet")
    a_variants = ["core", "proportional", "shin", "liquidity", "movement", "pin_only", "fund_market"]
    a_info = {}
    for name in BASES:
        base = pd.Series(np.nan, index=m.index)
        base.iloc[rows] = base_pred[name]["variant"].to_numpy()
        for v in a_variants:
            Z = fusion.stage2_inputs(m, s1[name], detail, v)
            p, info = fusion.stage2(m, Z, base)
            pv = p.iloc[rows].to_numpy()
            key = f"{name} + A[{v}]"
            preds[key] = pv
            a_info[key] = {"fallback_events": len(info["fallback_events"]),
                           "coef_last": list(info["coef"].values())[-1] if info["coef"] else None}
            pb = base_pred[name]["variant"].to_numpy()
            if name == "lgbm-ext" and v == "core":
                res["primary"] = _summ(key + " (market subset)", pv, pb, ref, mk)
                res["primary"]["fallback_events"] = len(info["fallback_events"])
                res["primary"]["stage2_rows"] = info["rows"]
                log(f"PRIMARY Δ log-loss {res['primary']['log_loss']['diff']:+.4f} "
                    f"[{res['primary']['log_loss']['low']:+.4f}, {res['primary']['log_loss']['high']:+.4f}]")
            res["rows"].append({**_summ(key, pv, pb, ref, mk), "subset": "market", "signal": "A", "base": name})
            res["rows"].append({**_summ(key, pv, pb, ref), "subset": "all", "signal": "A", "base": name})
        # stage 1 alone (no market) on all scored Matches, for reference
        p1 = s1[name].iloc[rows].to_numpy()
        p1 = np.where(np.isnan(p1), base_pred[name]["variant"].to_numpy(), p1)
        preds[f"{name} stage 1 (no market)"] = p1
    res["notes"]["A_info"] = a_info

    # de-vig comparison as standalone forecasts (Pinnacle rows)
    det = detail.reindex(ref["match_id"].to_numpy())
    from .market_detail import DEVIG
    pin = det["pin_oa"].notna().to_numpy()
    y = ref["team_a_won"].to_numpy()
    dv = {}
    for k, fn in DEVIG.items():
        p = np.array([fn(a, b) if ok else np.nan for a, b, ok in zip(det.pin_oa, det.pin_ob, pin)])
        dv[k] = {"log_loss": float(stats.log_loss(p[pin], y[pin]).mean()), "brier": float(stats.brier(p[pin], y[pin]).mean())}
        dv[k]["vs_proportional"] = None
        dv[k]["p"] = p
    for k in DEVIG:
        b = stats.event_bootstrap(ref["event_id"].to_numpy()[pin],
                                  stats.log_loss(dv[k]["p"][pin], y[pin]) - stats.log_loss(dv["proportional"]["p"][pin], y[pin]))
        dv[k]["vs_proportional"] = b
    for k in DEVIG:
        del dv[k]["p"]
    res["notes"]["devig"] = {"matches": int(pin.sum()), **dv}
    overround = (1 / det.pin_oa + 1 / det.pin_ob)[pin]
    res["notes"]["pinnacle_overround"] = float(overround.mean())

    # CLV-style: does the line move toward the fundamental model's early disagreement?
    res["notes"]["clv"] = clv(ref, det, s1, rows)

    # ---------------- B, C, D: feature add-ons
    for name, (model, fset) in BASES.items():
        pb = base_pred[name]["variant"].to_numpy()
        adds = [("B", "veto_eb", ["veto_eb"], ()), ("C", "openskill", ["openskill"], ()),
                ("D", "travel", ["travel"], ())]
        if "roster" in features.SETS[fset]:
            adds.append(("C", "openskill − roster", ["openskill"], ("roster",)))
        for sig, label, add, drop in adds:
            t = time.time()
            cols = _cols(fset, kinds, add, drop)
            pred = evaluate.walk_forward(m, X, cols, model, train_blocks=fusion.TRAIN_BLOCKS, log=lambda *_: None)
            pv = pred["variant"].to_numpy()
            key = f"{name} + {label}"
            preds[key] = pv
            res["rows"].append({**_summ(key, pv, pb, ref), "subset": "all", "signal": sig, "base": name})
            res["rows"].append({**_summ(key, pv, pb, ref, mk), "subset": "market", "signal": sig, "base": name})
            if sig == "B":
                bo3 = (m["format"].iloc[rows] == "bo3").to_numpy()
                res["rows"].append({**_summ(key, pv, pb, ref, bo3), "subset": "bo3", "signal": sig, "base": name})
            log(f"{key}: Δ ll {res['rows'][-1]['log_loss']['diff']:+.4f} [{time.time() - t:.0f}s]")
        for vname in VETO_VARIANTS:
            cols = _cols(fset, kinds)
            cols.update({f"{c}__{vname}": k for c, k in fs.VetoEB.columns.items()})
            pred = evaluate.walk_forward(m, X, cols, model, train_blocks=fusion.TRAIN_BLOCKS, log=lambda *_: None)
            key = f"{name} + {vname}"
            preds[key] = pred["variant"].to_numpy()
            res["rows"].append({**_summ(key, preds[key], pb, ref), "subset": "all", "signal": "B", "base": name})

    # B standalone against the anchor (as the lab's bo3_veto_sim)
    res["notes"]["veto_standalone"] = veto_standalone(ref, X, m, rows)
    res["notes"]["pool_changes"] = pool_changes(m, maps, hltv)
    res["notes"]["travel_cover"] = float((X.loc[ref["match_id"], "travel_has"] > 0).mean())
    res["notes"]["openskill_cover"] = float((X.loc[ref["match_id"], "os_has"] > 0).mean())
    res["notes"]["veto_cover_bo3"] = float((X.loc[ref["match_id"], "veb_has"] > 0).sum())

    out = ref[["match_id", "event_id", "team_a_won", "anchor", "has_market"]].copy()
    for k, v in preds.items():
        out[k] = v
    out.to_parquet(cache / "signals" / "preds.parquet", index=False)
    res["seconds"] = round(time.time() - t0)
    res["notes"]["timings"] = TIMINGS
    (cache / "signals" / "results.json").write_text(json.dumps(res, indent=1, default=float))
    return res


def clv(ref, det, s1, rows):
    """Early price = the first source of the chain order (Pinnacle power, Polymarket, Kalshi) that has
    both an early and a closing price. CLV_i = sign(z_model − z_early) · (z_close − z_early): positive
    when the line moved toward where the model disagreed with the early price."""
    from .market_detail import source_logits
    zc = source_logits(det, "power")
    ze = source_logits(det, "power", early=True)
    close = np.full(len(det), np.nan)
    early = np.full(len(det), np.nan)
    for s in ("kal", "poly", "pin"):  # last assignment wins: chain order
        ok = zc[f"z_{s}"].notna().to_numpy() & ze[f"z_{s}"].notna().to_numpy()
        close[ok] = zc[f"z_{s}"].to_numpy()[ok]
        early[ok] = ze[f"z_{s}"].to_numpy()[ok]
    ok = ~np.isnan(close)
    out = {"matches": int(ok.sum())}
    ev = ref["event_id"].to_numpy()
    y = ref["team_a_won"].to_numpy()
    for name, s in s1.items():
        zm = logit(np.clip(s.iloc[rows].to_numpy(), 1e-6, 1 - 1e-6))
        use = ok & ~np.isnan(s.iloc[rows].to_numpy())
        move = close - early
        d = np.sign(zm - early) * move
        moved = use & (np.abs(move) > 1e-9)
        b = stats.event_bootstrap(ev[use], d[use])
        toward = stats.event_bootstrap(ev[moved], (d[moved] > 0).astype(float))
        # does the early-vs-model disagreement also predict the result? (log-loss of early price vs close)
        ll_e = stats.log_loss(1 / (1 + np.exp(-early[use])), y[use])
        ll_c = stats.log_loss(1 / (1 + np.exp(-close[use])), y[use])
        out[name] = {"mean_clv_logit": b, "share_toward_model": toward, "moved": int(moved.sum()),
                     "close_minus_early_logloss": stats.event_bootstrap(ev[use], ll_c - ll_e)}
    return out


def veto_standalone(ref, X, m, rows):
    """Bo3 Matches with a simulated veto: the EB simulation alone, and blended with the market as the
    anchor blends the Valve rating (0.8 market + 0.2 simulation in logit space; the simulation alone
    without a market), against the anchor's picks."""
    xv = X.loc[ref["match_id"]]
    has = (xv["veb_has"] > 0).to_numpy()
    y = ref["team_a_won"].to_numpy()
    ev = ref["event_id"].to_numpy()
    a = ref["anchor"].to_numpy()
    zs = xv["veb_series_logit"].to_numpy()
    mkt = m["market_p"].iloc[rows].to_numpy(float)
    blend = np.where(np.isnan(mkt), zs, 0.8 * logit(np.nan_to_num(mkt, nan=0.5)) + 0.2 * zs)
    out = {"matches": int(has.sum())}
    for name, z in (("simulation", zs), ("market blend", blend)):
        p = 1 / (1 + np.exp(-z))
        out[name] = compare(p[has], a[has], y[has], ev[has])
    for vname in VETO_VARIANTS:
        z = xv[f"veb_series_logit__{vname}"].to_numpy()
        out[f"simulation {vname}"] = compare((1 / (1 + np.exp(-z)))[has], a[has], y[has], ev[has])
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(prog="kbml.signals")
    ap.parse_args(argv)
    try:
        sys.stdout.reconfigure(encoding="utf-8", line_buffering=True)
    except AttributeError:
        pass
    run()


if __name__ == "__main__":
    main()
