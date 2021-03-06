// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::fmt::{self, Write};
use std::iter::repeat;

use KenKen;

/// Represents a square sized table of some value type.
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

/// Function to display a (finished) puzzle solution.
impl fmt::Display for Tbl<u32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sep = repeat("+---").take(self.0).collect::<String>() + "+\n";
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

/// Represents a set of values (32 bits means we can handle values 0...31).
///
/// Since the set is used for candidate numbers, and the puzzle size is
/// restricted to 15, we don't need more space.
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

/// Represents a mask for numbers in rows and columns, used to check if we can insert
/// a number in a certain cell.
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

/// A small vector of up to 15 4-bit elements, represented as an u64.  The last
/// 4 bits hold the number of elements.
///
/// This is used to hold candidate sequences for cages.  This means we can have
/// cages with up to 15 cells, and the numbers must be <= 15 too.
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


pub fn format_square<T: fmt::Display>(ken: &KenKen, cellsize: usize, contents: &[T]) -> String {
    let mut res = String::with_capacity((cellsize + 1) * (ken.size + 2));
    let max = ken.size - 1;
    let cn = |i, j| if i <= max && j <= max { ken.cell2cage.get(i, j).0 } else { !0 };
    let cs = |s| repeat(s).take(cellsize).collect::<String>();
    res.push('┏');
    for j in 0..ken.size {
        res.push_str(&cs('━'));
        if j < max {
            res.push(if cn(0, j) != cn(0, j+1) { '┳' } else { '┯' });
        } else {
            res.push_str("┓\n");
        }
    }
    for i in 0..ken.size {
        res.push('┃');
        for j in 0..ken.size {
            write!(&mut res, "{0:^1$}", contents[i*ken.size + j], cellsize).unwrap();
            res.push(if cn(i, j) != cn(i, j+1) { '┃' } else { '│' });
        }
        res.push('\n');
        if i < max {
            res.push(if cn(i, 0) != cn(i+1, 0) { '┣' } else { '┠' });
            for j in 0..ken.size {
                let a = cn(i, j);
                let b = cn(i, j+1);
                let c = cn(i+1, j);
                let d = cn(i+1, j+1);
                res.push_str(&cs(if a != c { '━' } else { '─' }));
                if j < max {
                    res.push(match () {
                        _ if a == b && b == c && c == d => '┼',
                        _ if a == b && c == d => '┿',
                        _ if a == c && b == d => '╂',
                        _ if a == b && b == c => '╆',
                        _ if a == b && b == d => '╅',
                        _ if a == c && c == d => '╄',
                        _ if b == c && c == d => '╃',
                        _ if a == b => '╈',
                        _ if a == c => '╊',
                        _ if b == d => '╉',
                        _ if c == d => '╇',
                        _           => '╋',
                    });
                } else {
                    res.push(if a != c { '┫' } else { '┨' });
                    res.push('\n');
                }
            }
        } else {
            res.push('┗');
            for j in 0..ken.size {
                res.push_str(&cs('━'));
                if j < max {
                    res.push(if cn(i, j) != cn(i, j+1) { '┻' } else { '┷' });
                } else {
                    res.push_str("┛\n");
                }
            }
        }
    }
    res
}
