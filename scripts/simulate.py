"""Run a market and save the price series.

python scripts/simulate.py out.csv steps=50000 seed=2 makers=2 value=0
"""

import sys

from lobsim import engine

if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    params = dict(a.split("=", 1) for a in sys.argv[2:])
    result = engine.call("simulate", out=sys.argv[1], **params)
    for key, value in result.items():
        if key != "rows":
            print(f"{key:<26}{value}")
