// KenKen puzzle solver, (c) 2016 Georg Brandl.

mod helpers;
mod constraints;

use std::collections::BTreeMap;
use std::error::Error;
use std::env::args;
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::time::Instant;
use helpers::{Tbl, RowColMask};
use constraints::Constraints;

/// Represents the arithmetic operation in a cage.
pub enum Op {
    Const(u32),
    Add(u32),
    Sub(u32),
    Mul(u32),
    Div(u32),
}

/// Represents a single cage in a puzzle.
pub struct Cage {
    /// List of cell coordinates that belong to the cage.
    cells: Vec<(usize, usize)>,
    /// Operation and goal value of the cage.
    operation: Op,
}

impl Cage {
    /// Creates a new cage.  The operation is initially Const() because it
    /// is either Const or read afterwards.
    fn new(val: u32) -> Cage {
        Cage { cells: Vec::with_capacity(6), operation: Op::Const(val) }
    }
}

/// Represents a complete puzzle.
pub struct KenKen {
    /// Size of the puzzle (number of cells is size*size).
    size: usize,
    /// All cages.
    cages: Vec<Cage>,
    /// Mapping of cell (row, col) to (cage index, index within cage's cells).
    cell2cage: Tbl<(usize, usize)>,
}

impl KenKen {
    /// Load a puzzle from a file.
    fn load(filename: &str) -> Result<KenKen, Box<Error>> {
        let file = try!(File::open(filename));
        let mut it = BufReader::new(file).lines().enumerate().peekable();
        let mut cells = BTreeMap::new();
        let size = it.peek().and_then(|r| r.1.as_ref().map(String::len).ok()).unwrap_or(0);
        if size < 2 || size > 15 {
            return Err(format!("kenken size must be < 16 (found {})", size).into());
        }
        let cell2cage = Tbl::square(size, (!0, 0));
        let mut ken = KenKen { size: size, cages: Vec::new(), cell2cage: cell2cage };
        // Read the puzzle cage definition (first part).
        for (row, line) in it.by_ref() {
            let line = try!(line);
            if line.is_empty() {
                break;
            }
            if line.len() != size {
                return Err(format!("unequal line lengths (expected {}, found {})",
                                   size, line.len()).into());
            }
            for (col, ch) in line.chars().enumerate() {
                let cage = if ch.is_numeric() {
                    let val = format!("{}", ch).parse().unwrap();
                    ken.cages.push(Cage::new(val));
                    ken.cell2cage.put(row, col, (ken.cages.len() - 1, 0));
                    ken.cages.last_mut().unwrap()
                } else {
                    cells.entry(ch).or_insert_with(|| Cage::new(0))
                };
                cage.cells.push((row, col));
            }
        }
        // Read the cage's operation definitions, one per line.
        for (_, line) in it {
            let line = try!(line);
            if line.is_empty() {
                break;
            }
            let parts = line.split(": ").collect::<Vec<_>>();
            if parts.len() != 2 || parts[0].len() != 1 {
                return Err(format!("invalid line with cage: {}", line).into());
            }
            let key = try!(parts[0].chars().nth(0).ok_or("missing char before :"));
            if !cells.contains_key(&key) {
                continue;
            }
            let cage = try!(cells.get_mut(&key).ok_or(format!("reference to undefined cell {}", key)));
            let i = parts[1].len();
            let goal = try!(parts[1][..i-1].parse()
                            .map_err(|_| format!("invalid number: {}", &parts[1][..i-1])));
            cage.operation = match &parts[1][i-1..i] {
                "+" => Op::Add(goal),
                "-" => Op::Sub(goal),
                "*" => Op::Mul(goal),
                "/" => Op::Div(goal),
                other => return Err(format!("invalid operator: {}", other).into()),
            };
        }
        // Check the cage definitions and add the cages to the puzzle.
        for (key, cage) in cells {
            match cage.operation {
                Op::Sub(_) | Op::Div(_) => if cage.cells.len() != 2 {
                    return Err(format!("sub/div cages must have 2 cells, not {}", cage.cells.len()).into());
                },
                Op::Const(goal) => if goal == 0 {
                    return Err(format!("found cage ({}) without defined goal", key).into());
                },
                _ => if cage.cells.len() < 2 || cage.cells.len() > 15 {
                    return Err(format!("add/mul cages must have less than 16 cells, not {}",
                                       cage.cells.len()).into());
                }
            }
            for (i, &(row, col)) in cage.cells.iter().enumerate() {
                ken.cell2cage.put(row, col, (ken.cages.len(), i));
            }
            ken.cages.push(cage);
        }
        Ok(ken)
    }

    /// Solve the puzzle (or return a failure string).
    fn solve(&self) -> Result<(u32, Tbl<u32>), &'static str> {
        fn inner(ken: &KenKen, cons: &Constraints, work: &mut Tbl<u32>, res: &mut Vec<Tbl<u32>>,
                 mask: &mut RowColMask, steps: &mut u32, cageidx: usize)
        {
            *steps += 1;

            // try to place each cage candidate in its cells
            'outer: for cand in cons.get_cage_candidates(cageidx) {
                // check if we can do it without duplicating numbers in rows/cols
                for (cellidx, el) in cand.iter().enumerate() {
                    let (row, col) = ken.cages[cageidx].cells[cellidx];
                    if !mask.ok(row, col, el) {
                        continue 'outer;
                    }
                }
                // if yes, do it
                for (cellidx, el) in cand.iter().enumerate() {
                    let (row, col) = ken.cages[cageidx].cells[cellidx];
                    work.put(row, col, el);
                    mask.clear(row, col, el);
                }
                // and recurse
                if cageidx < ken.cages.len() - 1 {
                    inner(ken, cons, work, res, mask, steps, cageidx + 1)
                } else {
                    res.push(work.clone());  // solution found!
                }
                // reset row/colmasks for our candidate
                for (cellidx, el) in cand.iter().enumerate() {
                    let (row, col) = ken.cages[cageidx].cells[cellidx];
                    mask.set(row, col, el);
                }
            }
            // reset the cells
            for &(row, col) in &ken.cages[cageidx].cells {
                work.put(row, col, 0);
            }
        }

        let mut cons = Constraints::empty(self);
        cons.determine_initial();
        while cons.reduce() { }

        let mut work = Tbl::square(self.size, 0);
        let mut res = Vec::new();
        let mut mask = RowColMask::new(self.size);
        let mut steps = 0;
        inner(self, &cons, &mut work, &mut res, &mut mask, &mut steps, 0);
        if res.len() > 1 {
            Err("found more than 1 solution")
        } else {
            res.pop().ok_or("found no solution").map(|res| (steps, res))
        }
    }
}

fn main() {
    let args: Vec<_> = args().skip(1).collect();
    let show_solution = args.len() == 1;
    for arg in args {
        let puzzle = match KenKen::load(&arg) {
            Err(e) => { println!("*** Error loading {}: {}", arg, e); continue; }
            Ok(puzzle) => puzzle
        };
        let start = Instant::now();
        let (steps, solution) = match puzzle.solve() {
            Err(e) => { println!("*** Error solving {}: {}", arg, e); continue; }
            Ok(res) => res
        };
        let took = start.elapsed();
        let took = took.as_secs() as f64 + 1e-9 * took.subsec_nanos() as f64;
        if show_solution { print!("{}", solution); }
        println!("{:-20} {:8} steps {:10.4} ms", arg, steps, took * 1000.);
    }
}
