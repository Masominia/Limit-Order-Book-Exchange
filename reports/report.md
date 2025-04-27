# LobSim report

Prices in ticks, one tick is one basis point (bp) of the starting price. Quantities in lots. Time in steps.

## 1. Matching engine

50,000 random limit orders, market orders and cancellations were sent to the book and to a list-scanning reference: 28,436 trades, 0 differences, 0 consistency failures.

## 2. Populations

|  | noise only | noise + maker | noise + maker + value | all four |
|---|---|---|---|---|
| spread, ticks | 1.258 | 1.254 | 1.516 | 1.519 |
| depth near the touch, lots | 426.503 | 476.459 | 366.602 | 364.913 |
| volume per step, lots | 0.827 | 0.824 | 1.429 | 1.456 |
| volatility, ticks per step | 0.139 | 0.131 | 0.215 | 0.218 |
| variance ratio, 50 steps | 0.561 | 0.594 | 1.165 | 1.182 |
| excess kurtosis | 1.270 | 0.884 | 3.544 | 4.543 |
| order-flow autocorrelation | -6.78e-03 | -4.04e-03 | 0.145 | 0.148 |
| distance from value, ticks | 101.253 | 76.053 | 5.378 | 5.461 |

Agents of the full population at the end of the run (inventory in lots, profit in ticks times lots per step):

| kind | inventory | profit per step |
|---|---|---|
| noise | 4,229 | -0.30 |
| maker | 1 | -0.10 |
| momentum | 0 | -0.10 |
| value | -4,230 | 0.50 |

## 3. Shape of the book

Average resting volume per side peaks 3 ticks from the mid price (38.2 lots) and is 26.2 lots at the first tick.

## 4. Impact of one market order

| size, lots | cost, bp | se | mid move at once, bp | se  | mid move after 200 steps, bp | se   | not filled |
|---|---|---|---|---|---|---|---|
| 5.000 | 0.843 | 8.90e-03 | 0.105 | 4.78e-03 | 0.045 | 0.102 | 0.000 |
| 10.000 | 0.959 | 9.53e-03 | 0.246 | 6.11e-03 | 0.062 | 0.103 | 0.000 |
| 20.000 | 1.191 | 0.011 | 0.431 | 7.24e-03 | 0.140 | 0.104 | 0.000 |
| 40.000 | 1.560 | 0.012 | 0.744 | 9.29e-03 | 0.340 | 0.108 | 0.000 |
| 80.000 | 2.189 | 0.017 | 1.313 | 0.014 | 0.680 | 0.112 | 0.000 |
| 120.000 | 2.776 | 0.021 | 1.908 | 0.019 | 1.017 | 0.116 | 1.000 |
| 160.000 | 3.380 | 0.026 | 2.613 | 0.026 | 1.238 | 0.116 | 6.000 |

Half spread 0.76 bp. Above it, the cost grows as size to the power 0.86 (sizes of 20 lots and more).

## 5. Execution schedules

Parent order of 200 lots over 200 steps, sides alternating, 4,000 independent markets per schedule. Cost is the implementation shortfall against the arrival mid price, risk is its standard deviation.

|  | cost, bp | standard error | risk, bp | child orders | filled |
|---|---|---|---|---|---|
| immediate | 4.050 | 0.023 | 1.438 | 1.056 | 1.000 |
| front-loaded, urgency 8 | 3.344 | 0.020 | 1.246 | 77.000 | 1.000 |
| front-loaded, urgency 4 | 3.061 | 0.026 | 1.674 | 119.000 | 1.000 |
| front-loaded, urgency 2 | 2.835 | 0.035 | 2.236 | 163.000 | 1.000 |
| equal slices | 2.820 | 0.043 | 2.732 | 200.000 | 1.000 |
| volume profile | 2.730 | 0.042 | 2.676 | 170.000 | 1.000 |
| 20% of volume | 3.230 | 0.054 | 3.407 | 46.256 | 1.000 |

## Validation

14 PASS, 0 WARN, 0 FAIL. See `validation.md`.
