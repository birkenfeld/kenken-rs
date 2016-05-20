// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::fmt;
use std::collections::BTreeSet;

#[derive(Clone)]
pub struct Tbl<T>(usize, Vec<T>);

impl<T> Tbl<T> {
    pub fn square(n: usize, t: T) -> Tbl<T> where T: Clone {
        Tbl(n, vec![t; n*n])
    }

    pub fn get(&self, (i, j): (usize, usize)) -> &T {
        &self.1[i*self.0 + j]
    }

    pub fn get_mut(&mut self, (i, j): (usize, usize)) -> &mut T {
        &mut self.1[i*self.0 + j]
    }

    pub fn put(&mut self, i: usize, j: usize, t: T) {
        self.1[i*self.0 + j] = t;
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

impl fmt::Display for Tbl<BTreeSet<u32>> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let v: Vec<_> = self.1.iter().map(|set| {
            set.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join("")
        }).collect();
        let mut sep1 = String::from("+");
        for _ in 0..self.0+2 { sep1.push('-'); }
        let mut sep = vec![sep1; self.0].join("");
        sep.push_str("+\n");
        for row in v.chunks(self.0) {
            try!(f.write_str(&sep));
            for cell in row {
                try!(write!(f, "| {0:1$} ", cell, self.0));
            }
            try!(f.write_str("|\n"));
        }
        f.write_str(&sep)
    }
}
