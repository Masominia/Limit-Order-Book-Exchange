"""Static figures for the report (matplotlib, PNG for the README)."""

from __future__ import annotations

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402

# Validated categorical order (fixed, never cycled) + recessive ink.
SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"]
SURFACE, INK, INK2, GRID = "#fcfcfb", "#0b0b0b", "#52514e", "#e4e3df"
NEUTRAL = "#8a8985"
CRITICAL = "#c4271a"

plt.rcParams.update(
    {
        "figure.facecolor": SURFACE,
        "axes.facecolor": SURFACE,
        "savefig.facecolor": SURFACE,
        "axes.edgecolor": GRID,
        "axes.labelcolor": INK2,
        "xtick.color": INK2,
        "ytick.color": INK2,
        "text.color": INK,
        "axes.grid": True,
        "grid.color": GRID,
        "grid.linewidth": 0.6,
        "axes.spines.top": False,
        "axes.spines.right": False,
        "font.size": 9,
        "axes.titlesize": 10,
        "axes.titleweight": "bold",
        "legend.frameon": False,
        "lines.linewidth": 1.6,
        "figure.dpi": 150,
        "axes.axisbelow": True,
    }
)


def _save(fig, path):
    fig.tight_layout()
    fig.savefig(path, bbox_inches="tight")
    plt.close(fig)


def overview(pop, prof, imp, half_spread, exponent, front, path):
    fig, ax = plt.subplots(2, 2, figsize=(10.5, 7.4))
    a = ax[0, 0]
    a.bar(prof[:, 0], prof[:, 1], width=0.8, color=SERIES[0])
    a.set(
        title="Average shape of the book",
        xlabel="distance from the mid price, ticks",
        ylabel="resting volume per side, lots",
    )

    a = ax[0, 1]
    rows = ["spread, ticks", "excess kurtosis", "distance from value, ticks"]
    x = np.arange(len(rows))
    w = 0.8 / pop.shape[1]
    for i, col in enumerate(pop.columns):
        a.bar(x + (i - (pop.shape[1] - 1) / 2) * w, pop.loc[rows, col], width=w * 0.9, color=SERIES[i], label=col)
    a.set_xticks(x, rows)
    a.set_yscale("log")
    a.set(title="What each kind of agent changes")
    a.legend()

    a = ax[1, 0]
    s = imp.index.to_numpy(float)
    a.errorbar(
        s, imp["cost, bp"], yerr=2 * imp["se"], fmt="o-", color=SERIES[0], ms=4, capsize=2, label="cost of the order"
    )
    a.errorbar(
        s,
        imp.iloc[:, 2],
        yerr=2 * imp.iloc[:, 3],
        fmt="s-",
        color=SERIES[1],
        ms=4,
        capsize=2,
        label="mid price move, at once",
    )
    a.errorbar(
        s,
        imp.iloc[:, 4],
        yerr=2 * imp.iloc[:, 5],
        fmt="^-",
        color=SERIES[2],
        ms=4,
        capsize=2,
        label="mid price move, later",
    )
    a.axhline(half_spread, color=NEUTRAL, lw=1, ls="--", label="half spread")
    a.set_xscale("log")
    a.set(title=f"Impact of one market order (exponent {exponent:.2f})", xlabel="size, lots", ylabel="basis points")
    a.legend()

    a = ax[1, 1]
    for i, (label, r) in enumerate(front.iterrows()):
        a.errorbar(
            r["risk, bp"],
            r["cost, bp"],
            yerr=2 * r["standard error"],
            fmt="o",
            color=SERIES[i % len(SERIES)],
            ms=6,
            capsize=3,
            label=label,
        )
    a.set(
        title="Execution schedules: cost against risk",
        xlabel="risk (standard deviation of the shortfall), bp",
        ylabel="mean shortfall, bp",
    )
    a.legend(fontsize=7)
    _save(fig, path)
