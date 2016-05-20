// KenKen puzzle solver, (c) 2016 Georg Brandl.
#![feature(iter_arith)]

mod helpers;
mod constraints;

use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader};
use std::fs::File;
use std::cmp::{max, min};
use helpers::Tbl;

pub enum Op {
    Const(u32),
    Add(u32),
    Sub(u32),
    Mul(u32),
    Div(u32),
}

pub struct Cage {
    cells: Vec<(usize, usize)>,
    lastcell: (usize, usize),
    operation: Op,
}

impl Cage {
    fn new(val: u32) -> Cage {
        Cage { cells: Vec::with_capacity(6), lastcell: (0, 0), operation: Op::Const(val) }
    }

    fn satisfied(&self, tbl: &Tbl<u32>) -> bool {
        match self.operation {
            Op::Const(c) => tbl.get(self.cells[0]) == &c,
            Op::Add(goal) => self.cells.iter().map(|&c| tbl.get(c)).sum::<u32>() == goal,
            Op::Mul(goal) => self.cells.iter().map(|&c| tbl.get(c)).product::<u32>() == goal,
            Op::Sub(goal) => {
                let v1 = tbl.get(self.cells[0]);
                let v2 = tbl.get(self.cells[1]);
                max(v1, v2) - min(v1, v2) == goal
            }
            Op::Div(goal) => {
                let v1 = tbl.get(self.cells[0]);
                let v2 = tbl.get(self.cells[1]);
                let (a, b) = (max(v1, v2), min(v1, v2));
                a % b == 0 && a / b == goal
            }
        }
    }

    fn add_cell(&mut self, row: usize, col: usize) {
        self.cells.push((row, col));
        if row > self.lastcell.0 || (row == self.lastcell.0 && col > self.lastcell.1) {
            self.lastcell = (row, col);
        }
    }
}

pub struct KenKen {
    size: usize,
    cages: Vec<Cage>,
    cell2cage: Tbl<(usize, usize)>,
}

impl KenKen {
    fn load(filename: &str) -> Result<KenKen, String> {
        let file = try!(File::open(filename).map_err(|e| format!("{}", e)));
        let mut it = BufReader::new(file).lines().enumerate().peekable();
        let mut cells = BTreeMap::new();
        let size = try!(try!(it.peek().ok_or("no lines in file")).1.as_ref()
                        .map_err(|e| format!("{}", e))).len();
        if size < 2 || size > 15 {
            return Err(format!("kenken size must be < 16 (found {})", size));
        }
        let cell2cage = Tbl::square(size, (!0, 0));
        let mut ken = KenKen { size: size, cages: Vec::new(), cell2cage: cell2cage };
        for (row, line) in it.by_ref() {
            let line = try!(line.map_err(|e| format!("{}", e)));
            if line.is_empty() {
                break;
            }
            if line.len() != size {
                return Err(format!("unequal line lengths (expected {}, found {})",
                                   size, line.len()));
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
                cage.add_cell(row, col);
            }
        }
        for (_, line) in it {
            let line = try!(line.map_err(|e| format!("{}", e)));
            if line.is_empty() {
                break;
            }
            let parts = line.split(": ").collect::<Vec<_>>();
            if parts.len() != 2 || parts[0].len() != 1 {
                return Err(format!("invalid line with cage: {}", line));
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
                other => return Err(format!("invalid operator: {}", other)),
            };
        }
        for (key, cage) in cells {
            match cage.operation {
                Op::Sub(_) | Op::Div(_) => if cage.cells.len() != 2 {
                    return Err(format!("sub/div cages must have 2 cells, not {}", cage.cells.len()));
                },
                Op::Const(goal) => if goal == 0 {
                    return Err(format!("found cage ({}) without defined goal", key));
                },
                _ => if cage.cells.len() < 2 || cage.cells.len() > 15 {
                    return Err(format!("add/mul cages must have less than 16 cells, not {}",
                                       cage.cells.len()));
                }
            }
            for (i, &(row, col)) in cage.cells.iter().enumerate() {
                ken.cell2cage.put(row, col, (ken.cages.len(), i));
            }
            ken.cages.push(cage);
        }
        Ok(ken)
    }

    fn constraint_satisfied(&self, work: &Tbl<u32>, row: usize, col: usize) -> bool {
        let cage = &self.cages[self.cell2cage.get((row, col)).0];
        if (row, col) == cage.lastcell {
            cage.satisfied(work)
        } else {
            true
        }
    }

    fn solve(&self) -> Result<(u32, Tbl<u32>), &'static str> {
        fn inner(ken: &KenKen, cons: &Tbl<BTreeSet<u32>>, work: &mut Tbl<u32>, res: &mut Vec<Tbl<u32>>,
                 rmask: &mut [u32], cmask: &mut [u32], steps: &mut u32, row: usize, col: usize)
        {
            *steps += 1;
            let pval = rmask[row] & cmask[col];

            // try to place each candidate number in a cell
            for &v in cons.get((row, col)) {
                // check if we can do it without duplicating numbers in rows/cols
                if pval & (1 << v) == 0 {
                    continue;
                }
                // if yes, do it
                work.put(row, col, v as u32);
                // check if cage constraints are still satisfied
                if ken.constraint_satisfied(work, row, col) {
                    rmask[row] &= !(1 << v);
                    cmask[col] &= !(1 << v);
                    // and recurse
                    if col < ken.size - 1 {
                        inner(ken, cons, work, res, rmask, cmask, steps, row, col + 1);
                    } else if row < ken.size - 1 {
                        inner(ken, cons, work, res, rmask, cmask, steps, row + 1, 0);
                    } else {
                        res.push(work.clone());  // solution found!
                    }
                    // reset row/colmasks for our cell
                    rmask[row] |= 1 << v;
                    cmask[col] |= 1 << v;
                }
            }
            // reset the cell
            work.put(row, col, 0);
        }

        let mut cons = constraints::empty_constraints(self);
        constraints::initial_constraints(self, &mut cons);
        while constraints::reduce_constraints(self, &mut cons) { }

        let mut work = Tbl::square(self.size, 0);
        let mut res = Vec::new();
        let mut rmask = vec![((1 << self.size) - 1) << 1; self.size];
        let mut cmask = vec![((1 << self.size) - 1) << 1; self.size];
        let mut steps = 0;
        inner(self, &cons, &mut work, &mut res, &mut rmask, &mut cmask, &mut steps, 0, 0);
        if res.len() > 1 {
            Err("found more than 1 solution")
        } else {
            res.pop().ok_or("found no solution").map(|res| (steps, res))
        }
    }
}

fn main() {
    use std::env::args;
    use std::time::Instant;
    let args: Vec<_> = args().skip(1).collect();
    let nargs = args.len();
    for arg in args {
        match KenKen::load(&arg) {
            Err(e) => println!("Error loading {}: {}", arg, e),
            Ok(puz) => {
                let i1 = Instant::now();
                match puz.solve() {
                    Err(e) => println!("Error solving {}: {}", arg, e),
                    Ok((steps, solution)) => {
                        let el = i1.elapsed();
                        if nargs == 1 {
                            print!("{}", solution);
                        }
                        let el = el.as_secs() as f64 * 1000. + el.subsec_nanos() as f64 / 1000000.;
                        println!("{:-14} {:8} steps {:10.4} ms", arg, steps, el);
                    }
                }
            }
        }
    }
}
