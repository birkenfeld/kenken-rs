// KenKen puzzle solver, (c) 2016 Georg Brandl.

use std::fmt;

#[derive(Clone)]
pub struct Tbl<T>(usize, Vec<T>);

impl<T> Tbl<T> {
    pub fn square(n: usize, t: T) -> Tbl<T> where T: Clone {
        Tbl(n, vec![t; n*n])
    }

    pub fn get(&self, (i, j): (usize, usize)) -> &T {
        &self.1[i*self.0 + j]
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
