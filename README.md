Make sure to use `cargo run --release`

# CPU

```
CPU:
12th Gen Intel(R) Core(TM) i7-12700K
Logical processors:	20
```

# Experiments

The experiment consists in running a "bit test and set" in a loop, either using
the memory operand variant of `bts` or a "manual" implementation.
We report the number of iterations performed per second per thread.

In each case I waited for the throughput numbers to stabilize and reported the
stable value and the maximum observed throughput.

## Randomized index

In this experiment, the threads check random bits in the bitmap.

```
1 thread manual
    108 M iter per sec per thread (max observed: 111 M)
1 threads bts (memory operand variant)
    102 M iter per sec per thread (max observed: 102 M)
20 threads manual
    Starts at 11 M iter per sec per thread (max observed: 11 M)
    Rises to  20 M iter per sec per thread (max observed: 20 M)
20 threads bts (memory operand variant)
    10 M iter per sec per thread (max observed: 10 M)
```

### Comments

The manual version throughput is less stable than the bts (memory operand
variant) numbers.

This difference may be due to the presence of a branch in the manual
implementation. As more bits in the bitmap get set, the branch becomes easier
to predict.

Also, as more bits get set, the write back happens less often, so there are
fewer cache invalidations.

## Fixed index

In this experiment, the index of the tested bit is always the same.
This makes the performance difference more obvious.

```
1 thread manual
    1614 M iter per sec per thread (max observed: 1620 M)
1 threads bts (memory operand variant)
    643 M iter per sec per thread (max observed: 665 M)
20 threads manual
    848 M iter per sec per thread (max observed: 884 M)
20 threads bts (memory operand variant)
    89 M iter per sec per thread (max observed: 97 M)
```

### Comments

This experiment is better to demonstrate the issue raised in the initial
question.
With the manual implementation, the slowdown (per thread) when going from 1 to
20 is `1614 / 643 = 1.9`.
With the `bts` (memory operand variant) it is `643 / 89 = 7.2`.
