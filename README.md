# KenKen solver in Rust

This is a solver for KenKen puzzles, which are similar to Sudoku, but have cages
of varying sizes whose cells must fulfill mathematical operations.

The solver employs a combined strategy of first reducing the candidates for each
cage, and each cell in a cage, and then searching the solution using
backtracking.  Combined with Rust this makes it pretty fast.

## Input files

There are several example puzzles in `examples`.  The input format should be
pretty obvious.  Here is a small example:

```
abbc
a2cc
ddef
4def

a: 1-
b: 3-
c: 36*
d: 7+
e: 2/
f: 2/
```

The first part is a description of the cages (the puzzle size is taken from the
first line).  Each cage has a corresponding character - numbers are reserved for
single-cell constant cages.  In the second part, each cage is mapped to its
arithmetic rule.

Puzzles are accepted up to size 15x15.
