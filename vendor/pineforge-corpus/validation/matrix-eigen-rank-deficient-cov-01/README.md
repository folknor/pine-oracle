# typed-matrix-probe-02-eigen-rank-deficient

## Gap
Cross-confirmed P0 (X7, C++#1/#2): `matrix.eigenvalues()` on rank-deficient covariance - NaN propagation through orders is a first-emission landmine.

## Expected behavior
- 3x3 covariance from collinear sources (`close`, `2*close`, `3*close`) is rank 1; smallest eigenvalue should be ~0 or `na`.
- `eigenOk` should be FALSE almost every bar; entries take the fallback path.
- No spurious orders from NaN comparisons (`na > 1e-9` must be false).
- >=10 closed trades via fallback EMA cross.

## TV capture notes
- 15m chart, ETH-USDT-USDT, same window as `data/ohlcv_ETH-USDT-USDT_15m.csv`.
- Manual TradingView export → `tv_trades.csv`.
- Compare entry comments: TV vs PineForge should both pick `L_fb` (fallback) on identical bars. Any `L_eig` entry is a NaN-handling divergence.
