# Magnifier dist probe 07 - volume_weighted ON

**Gap targeted**: X3 (C++ #11) - magnifier volume-weighted code path (zero coverage).

**Expected behavior**: Parity-anomaly. Pine v6 has no keyword to toggle volume_weighted; harness must call strategy_set_magnifier_volume_weighted(true) before run(). TV does not expose the same toggle so divergence vs TV trades is expected.

**TV capture notes**:
- TV trades.csv exported with magnifier ON, volume_weighted OFF (TV default).\n- Harness re-runs with volume_weighted=true; diff that run against TV export.
