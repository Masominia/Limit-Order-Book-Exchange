# Validation checks

| area | check | status | detail |
|---|---|---|---|
| matching | identical fills to the reference book | PASS | 50,000 random orders, 28,436 trades, 0 mismatches |
| matching | internal consistency after every order | PASS | 0 failures |
| market | book consistent, positions and cash net to zero, both sides quoted | PASS | all populations |
| market | value traders tie the price to the fundamental value | PASS | distance 76.1 ticks without, 5.4 with |
| market | the full population has fat tails and persistent order flow | PASS | excess kurtosis 4.54, autocorrelation 0.15 (noise only: -0.01) |
| market | depth peaks away from the best price | PASS | largest average depth 3 ticks from the mid price |
| impact | cost increases with size | PASS | 0.84 to 3.38 bp |
| impact | smallest order pays about half the spread | PASS | 0.84 bp against 0.76 |
| impact | impact is concave in size | PASS | cost above the half spread grows as size to the power 0.86 |
| impact | part of the largest impact reverts | PASS | 47% of the immediate move remains after 200 steps |
| execution | every schedule completes the order | PASS | lowest fill rate 1.0000 |
| execution | slicing is cheaper than trading at once | PASS | 2.82 against 4.05 bp, 25.2 standard errors |
| execution | slicing is riskier than trading at once | PASS | 2.73 against 1.44 bp |
| execution | risk rises as urgency falls | PASS | 1.25, 1.67, 2.24, 2.73 |
