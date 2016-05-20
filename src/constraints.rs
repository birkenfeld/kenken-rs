// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::fmt;
use std::cmp::min;
use std::collections::BTreeSet;

use {KenKen, Cage, Op};
use helpers::Tbl;

struct CageCandidates(Vec<Vec<u32>>);

impl CageCandidates {
    fn from_cage(ken: &KenKen, cage: &Cage) -> CageCandidates {
        let size = ken.size as u32;
        let ncells = cage.cells.len();
        match cage.operation {
            Op::Add(goal) => CageCandidates(Self::for_add(size, goal, ncells)).reduced(cage),
            Op::Mul(goal) => CageCandidates(Self::for_mul(size, goal, ncells)).reduced(cage),
            Op::Sub(goal) => CageCandidates(Self::for_sub(size, goal)),
            Op::Div(goal) => CageCandidates(Self::for_div(size, goal)),
            Op::Const(c)  => CageCandidates(vec![vec![c]]),
        }
    }

    fn reduced(mut self, cage: &Cage) -> CageCandidates {
        for (i, &(row1, col1)) in cage.cells.iter().enumerate() {
            for (j, &(row2, col2)) in cage.cells.iter().enumerate().skip(i + 1) {
                self.0.retain(|cand| {
                    if row1 == row2 || col1 == col2 {
                        cand[i] != cand[j]
                    } else {
                        true
                    }
                });
            }
        }
        self
    }

    fn candidates_for_cell(&self, ix: usize) -> BTreeSet<u32> {
        self.0.iter().map(|v| v[ix]).collect()
    }

    fn for_add(max: u32, goal: u32, total: usize) -> Vec<Vec<u32>> {
        fn inner(max: u32, total: usize, len: u32, goal: u32) -> Vec<Vec<u32>> {
            if len == 1 {
                if goal <= max {
                    let mut v = Vec::with_capacity(total);
                    v.push(goal);
                    vec![v]
                } else {
                    vec![]
                }
            } else {
                let mut all = Vec::new();
                for i in 1..min(max + 1, goal - len + 2) {
                    let mut candidates = inner(max, total, len - 1, goal - i);
                    for v in &mut candidates {
                        v.push(i);
                    }
                    all.extend(candidates)
                }
                all
            }
        }
        inner(max, total, total as u32, goal)
    }

    fn for_mul(max: u32, goal: u32, total: usize) -> Vec<Vec<u32>> {
        fn inner(max: u32, total: usize, len: u32, goal: u32) -> Vec<Vec<u32>> {
            if len == 1 {
                if goal <= max {
                    let mut v = Vec::with_capacity(total);
                    v.push(goal);
                    vec![v]
                } else {
                    vec![]
                }
            } else {
                let mut all = Vec::new();
                for i in 1..min(max + 1, goal + 1) {
                    if goal % i != 0 {
                        continue;
                    }
                    let mut candidates = inner(max, total, len - 1, goal / i);
                    for v in &mut candidates {
                        v.push(i);
                    }
                    all.extend(candidates)
                }
                all
            }
        }
        inner(max, total, total as u32, goal)
    }

    fn for_sub(max: u32, goal: u32) -> Vec<Vec<u32>> {
        (1..max-goal+1).flat_map(|i| vec![vec![i, i + goal], vec![i + goal, i]]).collect()
    }

    fn for_div(max: u32, goal: u32) -> Vec<Vec<u32>> {
        (1..max/goal+1).flat_map(|i| vec![vec![i, i * goal], vec![i * goal, i]]).collect()
    }
}

pub struct Constraints<'a> {
    ken: &'a KenKen,
    cellcands: Tbl<BTreeSet<u32>>,
    cagecands: Vec<CageCandidates>,
}

impl<'a> Constraints<'a> {
    pub fn empty(ken: &'a KenKen) -> Constraints<'a> {
        Constraints {
            ken: ken,
            cellcands: Tbl::square(ken.size, (1..ken.size as u32 + 1).collect()),
            cagecands: Vec::with_capacity(ken.cages.len()),
        }
    }

    pub fn get_cage_candidates(&self, idx: usize) -> &Vec<Vec<u32>> {
        &self.cagecands[idx].0
    }

    fn get(&self, row: usize, col: usize) -> &BTreeSet<u32> {
        self.cellcands.get(row, col)
    }

    fn exclude(&mut self, row: usize, col: usize, el: u32) -> bool {
        let changed = self.cellcands.get_mut(row, col).remove(&el);
        if changed {
            let (cageidx, cellidx) = *self.ken.cell2cage.get(row, col);
            let rcands = &mut self.cagecands[cageidx];
            rcands.0.retain(|cand| cand[cellidx] != el);
            for (otheridx, &(row, col)) in self.ken.cages[cageidx].cells.iter().enumerate() {
                if otheridx != cellidx {
                    self.cellcands.put(row, col, rcands.candidates_for_cell(otheridx));
                }
            }
        }
        changed
    }

    pub fn determine_initial(&mut self) {
        for cage in &self.ken.cages {
            let new = CageCandidates::from_cage(self.ken, cage);
            for (cellidx, &(row, col)) in cage.cells.iter().enumerate() {
                self.cellcands.put(row, col, new.candidates_for_cell(cellidx));
            }
            self.cagecands.push(new);
        }
    }

    pub fn reduce(&mut self) -> bool {
        let mut changed = false;
        for row in 0..self.ken.size {
            for col in 0..self.ken.size {
                let n = self.get(row, col).len();
                // remove known values from other cells in same row/col
                if n == 1 {
                    let el = *self.get(row, col).iter().next().unwrap();
                    for other in 0..self.ken.size {
                        if other != col {
                            changed |= self.exclude(row, other, el);
                        }
                        if other != row {
                            changed |= self.exclude(other, col, el);
                        }
                    }
                }
                // remove values from other cells in same row/col if two cells are
                // known to have the same two possibilities
                else if n == 2 {
                    let els: Vec<_> = self.get(row, col).iter().cloned().collect();
                    for scol in col+1..self.ken.size {
                        if self.get(row, col) == self.get(row, scol) {
                            for ocol in 0..self.ken.size {
                                if ocol != col && ocol != scol {
                                    changed |= self.exclude(row, ocol, els[0]);
                                    changed |= self.exclude(row, ocol, els[1]);
                                }
                            }
                        }
                    }
                    for srow in row+1..self.ken.size {
                        if self.get(row, col) == self.get(srow, col) {
                            for orow in 0..self.ken.size {
                                if orow != row && orow != srow {
                                    changed |= self.exclude(orow, col, els[0]);
                                    changed |= self.exclude(orow, col, els[1]);
                                }
                            }
                        }
                    }
                }
            }
        }
        changed
    }
}

impl<'a> fmt::Display for Constraints<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.cellcands.fmt(f)
    }
}
