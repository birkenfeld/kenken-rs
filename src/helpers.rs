// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::fmt;

#[derive(Clone)]
pub struct Tbl<T>(usize, Vec<T>);

impl<T> Tbl<T> {
    pub fn square(n: usize, t: T) -> Tbl<T> where T: Clone {
        Tbl(n, vec![t; n*n])
    }

    pub fn get(&self, i: usize, j: usize) -> &T {
        &self.1[i*self.0 + j]
    }

    pub fn get_mut(&mut self, i: usize, j: usize) -> &mut T {
        &mut self.1[i*self.0 + j]
    }

    pub fn put(&mut self, i: usize, j: usize, t: T) {
        self.1[i*self.0 + j] = t;
    }

    pub fn as_vec(&self) -> &Vec<T> {
        &self.1
    }
}

impl fmt::Display for Tbl<u32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sep = vec!["+---"; self.0].join("");
        sep.push_str("+\n");
        for row in self.1.chunks(self.0) {
            try!(f.write_str(&sep));
            for cell in row {
                try!(write!(f, "| {} ", cell));
            }
            try!(f.write_str("|\n"));
        }
        f.write_str(&sep)
    }
}

#[derive(Clone, PartialEq)]
pub struct BitSet(u32);

impl BitSet {
    pub fn new_empty() -> BitSet {
        BitSet(0)
    }

    pub fn new_full(size: usize) -> BitSet {
        BitSet((1 << (size + 1)) - 2)
    }

    pub fn test(&self, bit: u32) -> bool {
        self.0 & (1 << bit) != 0
    }

    pub fn set(&mut self, bit: u32) {
        self.0 |= 1 << bit;
    }

    pub fn clear(&mut self, bit: u32) {
        self.0 &= !(1 << bit);
    }

    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    pub fn get_one(&self) -> u32 {
        self.0.trailing_zeros()
    }

    pub fn get_two(&self) -> (u32, u32) {
        (32 - self.0.leading_zeros() - 1, self.0.trailing_zeros())
    }

    pub fn to_string(&self) -> String {
        let mut res = String::new();
        for i in 1..10 {
            if self.test(i) {
                res.push_str(&format!("{}", i));
            }
        }
        res
    }
}

pub struct RowColMask(Vec<BitSet>, Vec<BitSet>);

impl RowColMask {
    pub fn new(size: usize) -> RowColMask {
        RowColMask(vec![BitSet::new_full(size); size], vec![BitSet::new_full(size); size])
    }

    pub fn ok(&self, row: usize, col: usize, el: u32) -> bool {
        self.0[row].test(el) && self.1[col].test(el)
    }

    pub fn set(&mut self, row: usize, col: usize, el: u32) {
        self.0[row].set(el);
        self.1[col].set(el);
    }

    pub fn clear(&mut self, row: usize, col: usize, el: u32) {
        self.0[row].clear(el);
        self.1[col].clear(el);
    }
}

#[derive(Clone)]
pub struct SmallVec(u64);

impl SmallVec {
    pub fn new_with(n: u32) -> SmallVec {
        SmallVec((1 << 60) | (n as u64))
    }

    pub fn new_with_two(n1: u32, n2: u32) -> SmallVec {
        SmallVec((2 << 60) | (n1 as u64) | (n2 as u64) << 4)
    }

    pub fn get(&self, ix: usize) -> u32 {
        (self.0 >> (ix << 2)) as u32 & 0xf
    }

    pub fn push(&mut self, n: u32) {
        let len = self.0 >> 60;
        let shift = len << 2;
        self.0 = (self.0 & ((1 << shift) - 1)) | (n as u64) << shift | (len + 1) << 60;
    }

    pub fn iter(&self) -> SmallVecIter {
        SmallVecIter(self.clone(), (self.0 >> 60) as usize, 0)
    }
}

pub struct SmallVecIter(SmallVec, usize, usize);

impl Iterator for SmallVecIter {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.2 < self.1 {
            self.2 += 1;
            Some(self.0.get(self.2 - 1))
        } else {
            None
        }
    }
}
