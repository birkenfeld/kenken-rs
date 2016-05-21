// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::fmt;
use std::iter::repeat;
use std::cmp::min;

use {KenKen, Cage, Op};
use helpers::{Tbl, BitSet, SmallVec};

/// Holds all candidate sequences for a single cage.
struct CageCandidates(Vec<SmallVec>);

impl CageCandidates {
    /// Generates initial candidates.
    fn from_cage(ken: &KenKen, cage: &Cage) -> CageCandidates {
        let size = ken.size as u32;
        let ncells = cage.cells.len() as u32;
        match cage.operation {
            Op::Add(goal) => CageCandidates(Self::for_add(size, goal, ncells)).reduced(cage),
            Op::Mul(goal) => CageCandidates(Self::for_mul(size, goal, ncells)).reduced(cage),
            Op::Sub(goal) => CageCandidates(Self::for_sub(size, goal)),
            Op::Div(goal) => CageCandidates(Self::for_div(size, goal)),
            Op::Const(c)  => CageCandidates(vec![SmallVec::new_with(c)]),
        }
    }

    /// Reduce the initial candidates by excluding candidates that have same
    /// numbers in a single row or column.
    fn reduced(mut self, cage: &Cage) -> CageCandidates {
        for (i, &(row1, col1)) in cage.cells.iter().enumerate() {
            for (j, &(row2, col2)) in cage.cells.iter().enumerate().skip(i + 1) {
                self.0.retain(|cand| {
                    if row1 == row2 || col1 == col2 {
                        cand.get(i) != cand.get(j)
                    } else {
                        true
                    }
                });
            }
        }
        self
    }

    /// Returns the possible numbers in a cell as a BitSet.
    fn candidates_for_cell(&self, ix: usize) -> BitSet {
        let mut res = BitSet::new_empty();
        for v in &self.0 {
            res.set(v.get(ix));
        }
        res
    }

    /// Generate possible sequences for an addition cage.
    fn for_add(max: u32, goal: u32, len: u32) -> Vec<SmallVec> {
        if len == 1 {
            if goal <= max {
                vec![SmallVec::new_with(goal)]
            } else {
                vec![]
            }
        } else {
            let mut all = Vec::new();
            for i in 1..min(max + 1, goal - len + 2) {
                let mut candidates = Self::for_add(max, goal - i, len - 1);
                for v in &mut candidates {
                    v.push(i);
                }
                all.extend(candidates)
            }
            all
        }
    }

    /// Generate possible sequences for a multiplication cage.
    fn for_mul(max: u32, goal: u32, len: u32) -> Vec<SmallVec> {
        if len == 1 {
            if goal <= max {
                vec![SmallVec::new_with(goal)]
            } else {
                vec![]
            }
        } else {
            let mut all = Vec::new();
            for i in 1..min(max + 1, goal + 1) {
                if goal % i != 0 {
                    continue;
                }
                let mut candidates = Self::for_mul(max, goal / i, len - 1);
                for v in &mut candidates {
                    v.push(i);
                }
                all.extend(candidates)
            }
            all
        }
    }

    /// Generate possible sequences for a subtraction cage.
    ///
    /// Subtraction cages always have two cells next to each other, so it is
    /// very easy to find the candidates.
    fn for_sub(max: u32, goal: u32) -> Vec<SmallVec> {
        (1..max-goal+1).flat_map(|i| vec![SmallVec::new_with_two(i, i + goal),
                                          SmallVec::new_with_two(i + goal, i)]).collect()
    }

    /// Generate possible sequences for a division cage.
    fn for_div(max: u32, goal: u32) -> Vec<SmallVec> {
        (1..max/goal+1).flat_map(|i| vec![SmallVec::new_with_two(i, i * goal),
                                          SmallVec::new_with_two(i * goal, i)]).collect()
    }
}

/// Represents all candidates for cages and individual cells for a single puzzle.
pub struct Constraints<'a> {
    ken: &'a KenKen,
    cellcands: Tbl<BitSet>,
    cagecands: Vec<CageCandidates>,
}

impl<'a> Constraints<'a> {
    pub fn empty(ken: &'a KenKen) -> Constraints<'a> {
        Constraints {
            ken: ken,
            cellcands: Tbl::square(ken.size, BitSet::new_full(ken.size)),
            cagecands: Vec::with_capacity(ken.cages.len()),
        }
    }

    pub fn get_cage_candidates(&self, idx: usize) -> &Vec<SmallVec> {
        &self.cagecands[idx].0
    }

    fn get(&self, row: usize, col: usize) -> &BitSet {
        self.cellcands.get(row, col)
    }

    /// Excludes number `el` from the cell at (row, col).  Updates cage candidates
    /// accordingly, and returns true if anything was changed.
    fn exclude(&mut self, row: usize, col: usize, el: u32) -> bool {
        if self.cellcands.get(row, col).test(el) {
            self.cellcands.get_mut(row, col).clear(el);

            let (cageidx, cellidx) = *self.ken.cell2cage.get(row, col);
            let rcands = &mut self.cagecands[cageidx];
            rcands.0.retain(|cand| cand.get(cellidx) != el);
            for (otheridx, &(row, col)) in self.ken.cages[cageidx].cells.iter().enumerate() {
                if otheridx != cellidx {
                    self.cellcands.put(row, col, rcands.candidates_for_cell(otheridx));
                }
            }
            true
        } else {
            false
        }
    }

    /// Determines initial constraints from cage candidates.
    pub fn determine_initial(&mut self) {
        for cage in &self.ken.cages {
            let new = CageCandidates::from_cage(self.ken, cage);
            for (cellidx, &(row, col)) in cage.cells.iter().enumerate() {
                self.cellcands.put(row, col, new.candidates_for_cell(cellidx));
            }
            self.cagecands.push(new);
        }
    }

    /// Tries to reduce constraints by removing known values within a
    /// row or column from other cages in that row or column.
    ///
    /// Returns true if anything was changed.
    pub fn reduce(&mut self) -> bool {
        let mut changed = false;
        for row in 0..self.ken.size {
            for col in 0..self.ken.size {
                let n = self.get(row, col).count();
                // remove known values from other cells in same row/col
                if n == 1 {
                    let el = self.get(row, col).get_one();
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
                    let (el1, el2) = self.get(row, col).get_two();
                    for scol in col+1..self.ken.size {
                        if self.get(row, col) == self.get(row, scol) {
                            for ocol in 0..self.ken.size {
                                if ocol != col && ocol != scol {
                                    changed |= self.exclude(row, ocol, el1);
                                    changed |= self.exclude(row, ocol, el2);
                                }
                            }
                        }
                    }
                    for srow in row+1..self.ken.size {
                        if self.get(row, col) == self.get(srow, col) {
                            for orow in 0..self.ken.size {
                                if orow != row && orow != srow {
                                    changed |= self.exclude(orow, col, el1);
                                    changed |= self.exclude(orow, col, el2);
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

/// Displays a table of candidates for each cell.
impl<'a> fmt::Display for Constraints<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let size = self.ken.size;
        let nums = &self.cellcands.as_vec();
        let v = nums.iter().map(|set| set.to_string()).collect::<Vec<_>>();
        let sep1 = "+".to_string() + &repeat('-').take(size+2).collect::<String>();
        let sep = repeat(&*sep1).take(size).collect::<String>() + "+\n";
        for row in v.chunks(size) {
            try!(f.write_str(&sep));
            for cell in row {
                try!(write!(f, "| {0:1$} ", cell, size));
            }
            try!(f.write_str("|\n"));
        }
        f.write_str(&sep)
    }
}
