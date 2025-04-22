import pytest

from lobsim import engine
from lobsim.pipeline import run


@pytest.mark.skipif(not engine.available(), reason="Rust engine not built")
def test_fast_pipeline(tmp_path):
    m = run(out=tmp_path, fast=True)
    assert not [c for c in m["checks"] if c["status"] == "FAIL"]
    assert (tmp_path / "report.md").exists() and (tmp_path / "figures" / "overview.png").exists()
    assert m["impact_exponent"] < 1.0
