"""Cost and risk of front-loaded schedules for a range of urgencies.

python scripts/frontier.py [size] [horizon] [seeds]
"""

import sys

from lobsim import engine

if __name__ == "__main__":
    size, horizon, seeds = (int(x) for x in (sys.argv[1:4] if len(sys.argv) >= 4 else (200, 200, 1000)))
    print(f"{'urgency':>8} {'cost, bp':>10} {'risk, bp':>10}")
    for urgency in (0, 0.5, 1, 2, 4, 8, 16):
        r = engine.call("execute", schedule="frontloaded", urgency=urgency, size=size, horizon=horizon, seeds=seeds)
        print(f"{urgency:>8} {r['mean_shortfall']:>10.2f} {r['std_shortfall']:>10.2f}")
