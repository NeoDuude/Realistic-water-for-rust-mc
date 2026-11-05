//! The MAC grid: velocity on faces, type per cell.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Air,
    Fluid,
    Solid,
}

/// Staggered velocity. `u` has one more sample along x than there are cells, and so on.
pub struct Mac {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub w: Vec<f32>,
    /// Weight sums from the particle transfer, one per face, kept to normalise and to know
    /// which faces got any particle (the rest are air and get extrapolated).
    pub wu: Vec<f32>,
    pub wv: Vec<f32>,
    pub ww: Vec<f32>,
}

impl Mac {
    pub fn new(nx: usize, ny: usize, nz: usize) -> Self {
        let u = vec![0.0; (nx + 1) * ny * nz];
        let v = vec![0.0; nx * (ny + 1) * nz];
        let w = vec![0.0; nx * ny * (nz + 1)];
        Mac {
            nx,
            ny,
            nz,
            wu: vec![0.0; u.len()],
            wv: vec![0.0; v.len()],
            ww: vec![0.0; w.len()],
            u,
            v,
            w,
        }
    }
    #[inline]
    pub fn ui(&self, i: usize, j: usize, k: usize) -> usize {
        (i) + (self.nx + 1) * (j + self.ny * k)
    }
    #[inline]
    pub fn vi(&self, i: usize, j: usize, k: usize) -> usize {
        (i) + self.nx * (j + (self.ny + 1) * k)
    }
    #[inline]
    pub fn wi(&self, i: usize, j: usize, k: usize) -> usize {
        (i) + self.nx * (j + self.ny * k)
    }
    #[inline]
    pub fn ci(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }
    pub fn clear(&mut self) {
        for a in [&mut self.u, &mut self.v, &mut self.w] {
            a.iter_mut().for_each(|x| *x = 0.0);
        }
        for a in [&mut self.wu, &mut self.wv, &mut self.ww] {
            a.iter_mut().for_each(|x| *x = 0.0);
        }
    }
}
