# Design and methods

## Book

Each side is an ordered map from price to a level, a level is a first-in first-out queue with a running total. A hash map sends every resting order id to its side and price, so a cancellation finds its level directly and then scans that one queue. Prices are integers, so there is no rounding anywhere in matching.

An incoming order is given an id, then swept against the opposite side: best price first, oldest order first, never past the limit price. Every fill is at the resting order's price. What is left of a limit order rests, what is left of a market order is dropped.

`Book::check` verifies that the book is not crossed, that no level is empty, that totals equal the sum of their queue and that the index matches the queues exactly.

## Reference book

`NaiveBook` keeps one unsorted list and scans it for the best opposite order at every fill. It shares no code with the fast book. The `fuzz` command and the `reference` test send the same random flow to both and require identical fills, return values and volumes after every single order.

## Market loop

One step:

1. the fundamental value takes a normal step,
2. each agent in turn sees the current book and returns actions, which are applied at once, so later agents see the effect of earlier ones,
3. fills are reported to the owners of both sides,
4. the mid price is stored, or the last one if a side is empty.

The run is a deterministic function of the parameters and the seed. Agent inventories and cash sum to zero when no outside order has been sent, which is tested.

## Agents

- **Noise.** Limit orders arrive at a fixed rate, at a distance from the opposite best price that is one tick plus an exponential variable. Market orders arrive at a second rate. Every resting order is cancelled with a fixed probability per step. Sizes are uniform.
- **Maker.** Every few steps it cancels its quotes and posts a ladder on both sides of a centre equal to the mid price minus a multiple of its inventory.
- **Momentum.** With some probability per step it sends a market order in the direction of the price change over a window, if that change exceeds a threshold.
- **Value.** It sends a market order towards the fundamental value with a probability proportional to the mispricing beyond a band.

## Statistics

Spread and depth are averages over steps. Volatility is the standard deviation of one-step changes of the mid price. The variance ratio divides the variance of non-overlapping changes over L steps by L times the one-step variance. Order-flow autocorrelation is the lag-one autocorrelation of the sign of net traded volume over the steps that traded.

## Impact

For each size, an independent market is warmed up, one market order is sent, and three numbers are recorded in basis points of the arrival mid price and signed so that adverse is positive: the average fill price, the mid price right after, and the mid price after a relaxation period. Buys and sells alternate so that any drift cancels.

## Execution

A schedule returns the cumulative quantity that should be done by the end of each step. The runner sends the difference to the current total as a market order after every market step; quantity that could not be filled is carried to the next step. Implementation shortfall is the average fill price against the arrival mid price.

The front-loaded schedule is the solution of the mean-variance problem with linear temporary impact: the remaining quantity is proportional to sinh(u (1 - t/T)), where u, the urgency, is the horizon times the square root of risk aversion times variance over the impact coefficient. Urgency zero gives equal slices.
