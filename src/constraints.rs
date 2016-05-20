// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::collections::BTreeSet;

use {KenKen, Op};
use helpers::Tbl;

fn generate_sums(max: u32, goal: u32, n: usize) -> BTreeSet<u32> {
    fn inner(max: u32, goal: u32, n: usize, s: &mut BTreeSet<u32>) {
        if n == 1 {
            if goal <= max { s.insert(goal); }
        } else {
            for i in 1..max+1 {
                if goal > i {
                    s.insert(i);
                    inner(max, goal - i, n - 1, s);
                }
            }
        }
    }
    let mut s = BTreeSet::new();
    inner(max, goal, n, &mut s);
    s
}

fn generate_subs(max: u32, goal: u32) -> BTreeSet<u32> {
    (1..max-goal+1).flat_map(|i| vec![i, i + goal]).collect()
}

fn generate_products(max: u32, goal: u32, n: usize) -> BTreeSet<u32> {
    fn inner(max: u32, goal: u32, n: usize, s: &mut BTreeSet<u32>) {
        if n == 1 {
            if goal <= max { s.insert(goal); }
        } else {
            for i in 1..max+1 {
                if goal % i == 0 {
                    s.insert(i);
                    inner(max, goal / i, n - 1, s);
                }
            }
        }
    }
    let mut s = BTreeSet::new();
    inner(max, goal, n, &mut s);
    s
}

fn generate_divs(max: u32, goal: u32) -> BTreeSet<u32> {
    (1..max/goal+1).flat_map(|i| vec![i, i * goal]).collect()
}

pub fn empty_constraints(ken: &KenKen) -> Tbl<BTreeSet<u32>> {
    Tbl::square(ken.size, (1..ken.size as u32 + 1).collect())
}

pub fn initial_constraints(ken: &KenKen, tbl: &mut Tbl<BTreeSet<u32>>) {
    let max = ken.size as u32;
    for cage in &ken.cages {
        let new = match cage.operation {
            Op::Const(c) => (c..c+1).collect(),
            Op::Add(goal) => generate_sums(max, goal, cage.cells.len()),
            Op::Sub(goal) => generate_subs(max, goal),
            Op::Mul(goal) => generate_products(max, goal, cage.cells.len()),
            Op::Div(goal) => generate_divs(max, goal),
        };
        for &(row, col) in &cage.cells {
            tbl.put(row, col, new.clone());
        }
    }
}

pub fn reduce_constraints(ken: &KenKen, tbl: &mut Tbl<BTreeSet<u32>>) -> bool {
    let mut changed = false;
    for row in 0..ken.size {
        for col in 0..ken.size {
            // remove known values from other cells in same row/col
            if tbl.get((row, col)).len() == 1 {
                let el = *tbl.get((row, col)).iter().next().unwrap();
                for other in 0..ken.size {
                    if other != col {
                        changed |= tbl.get_mut((row, other)).remove(&el);
                    }
                    if other != row {
                        changed |= tbl.get_mut((other, col)).remove(&el);
                    }
                }
            }
        }
    }
    for row in 0..ken.size {
        for col in 0..ken.size {
            if tbl.get((row, col)).len() == 2 {
                for second in col+1..ken.size {
                    if tbl.get((row, col)) == tbl.get((row, second)) {
                        for other in 0..ken.size {
                            if other == col || other == second {
                                continue;
                            }
                            let new = tbl.get((row, other)).difference(&tbl.get((row, col))).cloned().collect();
                            if &new != tbl.get((row, other)) {
                                changed = true;
                                tbl.put(row, other, new);
                            }
                        }
                    }
                }
                for second in row+1..ken.size {
                    if tbl.get((row, col)) == tbl.get((second, col)) {
                        for other in 0..ken.size {
                            if other == row || other == second {
                                continue;
                            }
                            let new = tbl.get((other, col)).difference(&tbl.get((row, col))).cloned().collect();
                            if &new != tbl.get((other, col)) {
                                changed = true;
                                tbl.put(other, col, new);
                            }
                        }
                    }
                }
            }
        }
    }
    changed
}
