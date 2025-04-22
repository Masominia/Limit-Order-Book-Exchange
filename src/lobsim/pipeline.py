"""All experiments, checks and the report."""

from __future__ import annotations

import json
from pathlib import Path

import numpy as np
import pandas as pd
import yaml

from . import engine, plotting
from .engine import ROOT
from .report import md_table


def _check(checks, area, name, ok, detail, warn=False):
    checks.append(
        {"area": area, "check": name, "status": "PASS" if ok else ("WARN" if warn else "FAIL"), "detail": detail}
    )


def run(cfg_path=None, out=None, fast: bool = False) -> dict:
    cfg = yaml.safe_load(open(cfg_path or ROOT / "configs" / "default.yaml"))
    out = Path(out) if out else ROOT / "reports"
    (out / "figures").mkdir(parents=True, exist_ok=True)
    if not engine.available():
        raise SystemExit("Rust engine not built: run `make build` first")
    seed, mkt, k = cfg["seed"], cfg["market"] or {}, (10 if fast else 1)
    checks: list[dict] = []

    # 1. the matching engine against a list-scanning reference, and its speed
    fz = engine.call("fuzz", orders=cfg["fuzz_orders"] // k, seed=seed)
    bench = engine.call("bench", orders=cfg["bench_orders"] // k, seed=seed)
    _check(
        checks,
        "matching",
        "identical fills to the reference book",
        fz["mismatches"] == 0,
        f"{fz['orders']:,.0f} random orders, {fz['trades']:,.0f} trades, {fz['mismatches']:.0f} mismatches",
    )
    _check(
        checks,
        "matching",
        "internal consistency after every order",
        fz["invariant_failures"] == 0,
        f"{fz['invariant_failures']:.0f} failures",
    )

    # 2. what each kind of agent does to the market
    keys = [
        "mean_spread",
        "mean_depth",
        "volume_per_step",
        "volatility",
        "variance_ratio",
        "kurtosis",
        "flow_autocorrelation",
        "mispricing",
    ]
    names = [
        "spread, ticks",
        "depth near the touch, lots",
        "volume per step, lots",
        "volatility, ticks per step",
        f"variance ratio, {cfg['lag']} steps",
        "excess kurtosis",
        "order-flow autocorrelation",
        "distance from value, ticks",
    ]
    pops, agents, sound = {}, None, True
    for label, over in cfg["populations"].items():
        s = engine.call("simulate", steps=cfg["simulation_steps"] // k, lag=cfg["lag"], seed=seed, **{**mkt, **over})
        pops[label] = {n: s[key] for n, key in zip(names, keys, strict=True)}
        sound &= s["check_failures"] == 0 and s["net_inventory"] == 0 and s["net_cash"] == 0 and s["one_sided"] < 0.01
        agents = s["rows"]
    pop = pd.DataFrame(pops)
    first, last = pop.iloc[:, 0], pop.iloc[:, -1]
    _check(
        checks,
        "market",
        "book consistent, positions and cash net to zero, both sides quoted",
        bool(sound),
        "all populations",
    )
    _check(
        checks,
        "market",
        "value traders tie the price to the fundamental value",
        pop.iloc[-1, 2] < 0.5 * pop.iloc[-1, 1],
        f"distance {pop.iloc[-1, 1]:.1f} ticks without, {pop.iloc[-1, 2]:.1f} with",
    )
    _check(
        checks,
        "market",
        "the full population has fat tails and persistent order flow",
        last["excess kurtosis"] > 0.5 and last["order-flow autocorrelation"] > 0.05,
        f"excess kurtosis {last['excess kurtosis']:.2f}, autocorrelation {last['order-flow autocorrelation']:.2f} (noise only: {first['order-flow autocorrelation']:.2f})",
        warn=fast,
    )

    # 3. average shape of the book
    prof = np.array(engine.call("profile", steps=cfg["profile_steps"] // k, seed=seed, **mkt)["rows"])
    peak = int(prof[np.argmax(prof[:, 1]), 0])
    _check(
        checks,
        "market",
        "depth peaks away from the best price",
        1 < peak < 15,
        f"largest average depth {peak} ticks from the mid price",
    )

    # 4. cost of a single market order against its size
    ic = cfg["impact"]
    sizes = ic["sizes"][: 5 if fast else None]
    rows = engine.call(
        "impact", sizes=",".join(map(str, sizes)), seeds=ic["seeds"] // k, relax=ic["relax"], seed=seed, **mkt
    )["rows"]
    imp = pd.DataFrame(
        rows,
        columns=[
            "size, lots",
            "cost, bp",
            "se",
            "mid move at once, bp",
            "se ",
            f"mid move after {ic['relax']} steps, bp",
            "se  ",
            "not filled",
        ],
    ).set_index("size, lots")
    half_spread = float(pop.iloc[0, -1]) / 2
    excess = imp["cost, bp"] - half_spread
    fit = imp.index >= 20
    exponent = float(np.polyfit(np.log(imp.index[fit]), np.log(excess[fit]), 1)[0])
    _check(
        checks,
        "impact",
        "cost increases with size",
        bool(np.all(np.diff(imp["cost, bp"]) > 0)),
        f"{imp['cost, bp'].iloc[0]:.2f} to {imp['cost, bp'].iloc[-1]:.2f} bp",
    )
    _check(
        checks,
        "impact",
        "smallest order pays about half the spread",
        abs(imp["cost, bp"].iloc[0] / half_spread - 1) < 0.25,
        f"{imp['cost, bp'].iloc[0]:.2f} bp against {half_spread:.2f}",
    )
    _check(
        checks,
        "impact",
        "impact is concave in size",
        exponent < 0.95,
        f"cost above the half spread grows as size to the power {exponent:.2f}",
    )
    decay = imp.iloc[-1, 4] / imp.iloc[-1, 2]
    _check(
        checks,
        "impact",
        "part of the largest impact reverts",
        decay < 0.9,
        f"{decay:.0%} of the immediate move remains after {ic['relax']} steps",
        warn=True,
    )

    # 5. execution schedules: cost against risk
    ex = cfg["execution"]
    fr = {}
    for label, args in ex["schedules"].items():
        r = engine.call(
            "execute", size=ex["size"], horizon=ex["horizon"], seeds=ex["seeds"] // k, seed=seed, **args, **mkt
        )
        fr[label] = {
            "cost, bp": r["mean_shortfall"],
            "standard error": r["std_error"],
            "risk, bp": r["std_shortfall"],
            "child orders": r["child_orders"],
            "filled": r["fill_rate"],
        }
    front = pd.DataFrame(fr).T
    now, even = front.loc["immediate"], front.loc["equal slices"]
    gap = (now["cost, bp"] - even["cost, bp"]) / np.hypot(now["standard error"], even["standard error"])
    _check(
        checks,
        "execution",
        "every schedule completes the order",
        bool((front["filled"] > 0.999).all()),
        f"lowest fill rate {front['filled'].min():.4f}",
    )
    _check(
        checks,
        "execution",
        "slicing is cheaper than trading at once",
        gap > 3,
        f"{even['cost, bp']:.2f} against {now['cost, bp']:.2f} bp, {gap:.1f} standard errors",
        warn=fast,
    )
    _check(
        checks,
        "execution",
        "slicing is riskier than trading at once",
        even["risk, bp"] > 1.3 * now["risk, bp"],
        f"{even['risk, bp']:.2f} against {now['risk, bp']:.2f} bp",
    )
    ladder = front.loc[
        [x for x in front.index if x.startswith("front-loaded")] + ["equal slices"], "risk, bp"
    ].to_numpy()
    _check(
        checks,
        "execution",
        "risk rises as urgency falls",
        bool(np.all(np.diff(ladder) > 0)),
        ", ".join(f"{x:.2f}" for x in ladder),
        warn=fast,
    )

    plotting.overview(pop, prof, imp, half_spread, exponent, front, out / "figures" / "overview.png")

    val = pd.DataFrame(checks)
    counts = val["status"].value_counts().to_dict()
    L = [
        "# LobSim report",
        "",
        "Prices in ticks, one tick is one basis point (bp) of the starting price. Quantities in lots. Time in steps.",
        "",
        "## 1. Matching engine",
        "",
        f"{fz['orders']:,.0f} random limit orders, market orders and cancellations were sent to the book and to a list-scanning reference: {fz['trades']:,.0f} trades, {fz['mismatches']:.0f} differences, {fz['invariant_failures']:.0f} consistency failures.",
        "",
        "## 2. Populations",
        "",
        md_table(pop, 3),
        "",
        "Agents of the full population at the end of the run (inventory in lots, profit in ticks times lots per step):",
        "",
        md_table(
            pd.DataFrame([[r[2], int(r[3]), r[4]] for r in agents], columns=["kind", "inventory", "profit per step"]),
            2,
            index=False,
        ),
        "",
        "## 3. Shape of the book",
        "",
        f"Average resting volume per side peaks {peak} ticks from the mid price ({prof[:, 1].max():.1f} lots) and is {prof[0, 1]:.1f} lots at the first tick.",
        "",
        "## 4. Impact of one market order",
        "",
        md_table(imp, 3),
        "",
        f"Half spread {half_spread:.2f} bp. Above it, the cost grows as size to the power {exponent:.2f} (sizes of 20 lots and more).",
        "",
        "## 5. Execution schedules",
        "",
        f"Parent order of {ex['size']} lots over {ex['horizon']} steps, sides alternating, {ex['seeds'] // k:,d} independent markets per schedule. Cost is the implementation shortfall against the arrival mid price, risk is its standard deviation.",
        "",
        md_table(front, 3),
        "",
        "## Validation",
        "",
        f"{counts.get('PASS', 0)} PASS, {counts.get('WARN', 0)} WARN, {counts.get('FAIL', 0)} FAIL. See `validation.md`.",
        "",
    ]
    (out / "report.md").write_text("\n".join(L))
    (out / "validation.md").write_text("# Validation checks\n\n" + md_table(val, index=False) + "\n")
    metrics = {
        "fuzz": {x: fz[x] for x in ("orders", "trades", "mismatches", "invariant_failures")},
        "populations": pop.to_dict(),
        "profile": prof.tolist(),
        "impact": imp.to_dict("index"),
        "impact_exponent": exponent,
        "frontier": front.to_dict("index"),
        "checks": checks,
    }
    (out / "metrics.json").write_text(json.dumps(metrics, indent=1, default=float))
    print(f"matching engine: about {bench['orders_per_second'] / 1e6:.1f} million orders per second")
    return metrics
