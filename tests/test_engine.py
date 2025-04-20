import pytest

from lobsim import engine

pytestmark = pytest.mark.skipif(not engine.available(), reason="Rust engine not built")


def test_fuzz_agrees_with_reference():
    r = engine.call("fuzz", orders=5000, seed=4)
    assert r["mismatches"] == 0 and r["invariant_failures"] == 0 and r["trades"] > 1000


def test_simulation_is_reproducible_and_sound():
    a = engine.call("simulate", steps=5000, seed=3, warmup=500)
    b = engine.call("simulate", steps=5000, seed=3, warmup=500)
    a.pop("seconds"), b.pop("seconds")
    assert a == b
    assert a["net_inventory"] == 0 and a["net_cash"] == 0 and a["check_failures"] == 0
    assert 1.0 <= a["mean_spread"] < 5.0 and len(a["rows"]) == 4


def test_series_file(tmp_path):
    path = tmp_path / "series.csv"
    engine.call("simulate", steps=2000, warmup=500, out=path, every=20)
    lines = path.read_text().splitlines()
    assert lines[0] == "step,mid,spread,depth,volume" and len(lines) == 101


def test_impact_grows_with_size():
    rows = engine.call("impact", sizes="5,80", seeds=200, relax=10, warmup=1000)["rows"]
    assert rows[1][1] > rows[0][1] + 3 * (rows[0][2] + rows[1][2])


def test_schedules_trade_cost_for_risk():
    kw = {"size": 150, "horizon": 100, "seeds": 300, "warmup": 1000}
    now, even = engine.call("execute", schedule="immediate", **kw), engine.call("execute", schedule="twap", **kw)
    assert now["fill_rate"] == 1.0 and even["fill_rate"] == 1.0
    assert even["mean_shortfall"] < now["mean_shortfall"] and even["std_shortfall"] > now["std_shortfall"]
    assert now["schedule"] == "immediate"
