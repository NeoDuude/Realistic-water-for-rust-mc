//! The FLIP/PIC step. One `step` is one frame of fluid.
use crate::grid::{Cell, Mac};

#[derive(Clone, Copy)]
pub struct Params {
    /// Grid cells per block. The GPU uses 4..8; tests use small grids.
    pub dx: f32,
    pub gravity: f32,
    /// 0 = pure PIC (smooth, loses energy), 1 = pure FLIP (lively, noisy). ~0.95 is usual.
    pub flip: f32,
    /// Pressure iterations (Gauss-Seidel red-black). More = stiffer, less squash.
    pub iters: u32,
    /// Push particles back under this many per cell, so they don't clump (volume drift guard).
    pub target_per_cell: f32,
}

impl Default for Params {
    fn default() -> Self {
        Params { dx: 1.0, gravity: 9.81, flip: 0.95, iters: 40, target_per_cell: 8.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub p: [f32; 3],
    pub v: [f32; 3],
}

pub struct Fluid {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub mac: Mac,
    old: Mac,
    pub cell: Vec<Cell>,
    /// Solid mask from the world (set by the caller each step).
    pub solid: Vec<bool>,
    pub pressure: Vec<f32>,
    div: Vec<f32>,
    pub particles: Vec<Particle>,
    pub prm: Params,
}

#[inline]
fn tri(t: f32) -> (f32, f32) {
    (1.0 - t, t)
}

impl Fluid {
    pub fn new(nx: usize, ny: usize, nz: usize, prm: Params) -> Self {
        let n = nx * ny * nz;
        Fluid {
            nx,
            ny,
            nz,
            mac: Mac::new(nx, ny, nz),
            old: Mac::new(nx, ny, nz),
            cell: vec![Cell::Air; n],
            solid: vec![false; n],
            pressure: vec![0.0; n],
            div: vec![0.0; n],
            particles: Vec::new(),
            prm,
        }
    }

    #[inline]
    fn ci(&self, i: usize, j: usize, k: usize) -> usize {
        i + self.nx * (j + self.ny * k)
    }
    #[inline]
    fn in_grid(&self, i: i32, j: i32, k: i32) -> bool {
        i >= 0 && j >= 0 && k >= 0 && (i as usize) < self.nx && (j as usize) < self.ny && (k as usize) < self.nz
    }

    /// Fill a box with particles, `n` per cell on a jittered lattice.
    pub fn fill_box(&mut self, lo: [usize; 3], hi: [usize; 3], per_cell: usize, seed: &mut u32) {
        for k in lo[2]..hi[2] {
            for j in lo[1]..hi[1] {
                for i in lo[0]..hi[0] {
                    for _ in 0..per_cell {
                        let r = |s: &mut u32| {
                            *s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                            (*s >> 8) as f32 / (1u32 << 24) as f32
                        };
                        self.particles.push(Particle {
                            p: [i as f32 + r(seed), j as f32 + r(seed), k as f32 + r(seed)],
                            v: [0.0; 3],
                        });
                    }
                }
            }
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.p2g();
        self.save_old();
        self.mark_cells();
        self.add_gravity(dt);
        self.set_solid_faces();
        self.solve_pressure(dt);
        self.set_solid_faces();
        self.extrapolate();
        self.g2p();
        self.advect(dt);
    }

    /// Splat particle velocity onto faces with trilinear weights.
    fn p2g(&mut self) {
        self.mac.clear();
        let (nx, ny, nz) = (self.nx, self.ny, self.nz);
        for pt in &self.particles {
            // u faces sit at (i, j+0.5, k+0.5).
            splat(&mut self.mac.u, &mut self.mac.wu, nx + 1, ny, nz, pt.p, [0.0, 0.5, 0.5], pt.v[0]);
            splat(&mut self.mac.v, &mut self.mac.wv, nx, ny + 1, nz, pt.p, [0.5, 0.0, 0.5], pt.v[1]);
            splat(&mut self.mac.w, &mut self.mac.ww, nx, ny, nz + 1, pt.p, [0.5, 0.5, 0.0], pt.v[2]);
        }
        norm(&mut self.mac.u, &self.mac.wu);
        norm(&mut self.mac.v, &self.mac.wv);
        norm(&mut self.mac.w, &self.mac.ww);
    }

    fn save_old(&mut self) {
        self.old.u.copy_from_slice(&self.mac.u);
        self.old.v.copy_from_slice(&self.mac.v);
        self.old.w.copy_from_slice(&self.mac.w);
    }

    fn mark_cells(&mut self) {
        for (c, s) in self.cell.iter_mut().zip(self.solid.iter()) {
            *c = if *s { Cell::Solid } else { Cell::Air };
        }
        for pt in &self.particles {
            let (i, j, k) = (pt.p[0] as i32, pt.p[1] as i32, pt.p[2] as i32);
            if self.in_grid(i, j, k) {
                let idx = self.ci(i as usize, j as usize, k as usize);
                if self.cell[idx] != Cell::Solid {
                    self.cell[idx] = Cell::Fluid;
                }
            }
        }
    }

    fn add_gravity(&mut self, dt: f32) {
        for v in &mut self.mac.v {
            *v -= self.prm.gravity * dt;
        }
    }

    /// Velocity into a solid, on the face between solid and non-solid, is zeroed (no-slip normal).
    fn set_solid_faces(&mut self) {
        let (nx, ny, nz) = (self.nx, self.ny, self.nz);
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..=nx {
                    let left = if i == 0 { true } else { self.solid[self.ci(i - 1, j, k)] };
                    let right = if i == nx { true } else { self.solid[self.ci(i, j, k)] };
                    if left || right {
                        let idx = self.mac.ui(i, j, k);
                        self.mac.u[idx] = 0.0;
                    }
                }
            }
        }
        for k in 0..nz {
            for j in 0..=ny {
                for i in 0..nx {
                    let down = if j == 0 { true } else { self.solid[self.ci(i, j - 1, k)] };
                    let up = if j == ny { true } else { self.solid[self.ci(i, j, k)] };
                    if down || up {
                        let idx = self.mac.vi(i, j, k);
                        self.mac.v[idx] = 0.0;
                    }
                }
            }
        }
        for k in 0..=nz {
            for j in 0..ny {
                for i in 0..nx {
                    let back = if k == 0 { true } else { self.solid[self.ci(i, j, k - 1)] };
                    let front = if k == nz { true } else { self.solid[self.ci(i, j, k)] };
                    if back || front {
                        let idx = self.mac.wi(i, j, k);
                        self.mac.w[idx] = 0.0;
                    }
                }
            }
        }
    }

    /// Divergence of the fluid cells, then a red-black Gauss-Seidel Poisson solve, then subtract
    /// the pressure gradient so each fluid cell's velocity has zero divergence: volume held.
    fn solve_pressure(&mut self, dt: f32) {
        let (nx, ny, nz) = (self.nx, self.ny, self.nz);
        let scale = dt / (self.prm.dx * self.prm.dx);
        for p in &mut self.pressure {
            *p = 0.0;
        }
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let c = self.ci(i, j, k);
                    if self.cell[c] != Cell::Fluid {
                        self.div[c] = 0.0;
                        continue;
                    }
                    let d = (self.mac.u[self.mac.ui(i + 1, j, k)] - self.mac.u[self.mac.ui(i, j, k)]
                        + self.mac.v[self.mac.vi(i, j + 1, k)] - self.mac.v[self.mac.vi(i, j, k)]
                        + self.mac.w[self.mac.wi(i, j, k + 1)] - self.mac.w[self.mac.wi(i, j, k)])
                        / self.prm.dx;
                    self.div[c] = -d;
                }
            }
        }
        let solid = |s: &Self, i: i32, j: i32, k: i32| !s.in_grid(i, j, k) || s.solid[s.ci(i as usize, j as usize, k as usize)];
        for _ in 0..self.prm.iters {
            for color in 0..2 {
                for k in 0..nz {
                    for j in 0..ny {
                        for i in 0..nx {
                            if (i + j + k) & 1 != color {
                                continue;
                            }
                            let c = self.ci(i, j, k);
                            if self.cell[c] != Cell::Fluid {
                                continue;
                            }
                            let (i0, j0, k0) = (i as i32, j as i32, k as i32);
                            let mut diag = 0.0f32;
                            let mut sum = self.div[c];
                            for (di, dj, dk) in [(-1, 0, 0), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)] {
                                let (ni, nj, nk) = (i0 + di, j0 + dj, k0 + dk);
                                if solid(self, ni, nj, nk) {
                                    continue; // Neumann: solid neighbour adds nothing.
                                }
                                diag += 1.0;
                                if self.in_grid(ni, nj, nk) {
                                    let nc = self.ci(ni as usize, nj as usize, nk as usize);
                                    if self.cell[nc] == Cell::Fluid {
                                        sum += self.pressure[nc];
                                    }
                                    // Air neighbour: pressure 0, adds nothing but counts in diag.
                                }
                            }
                            if diag > 0.0 {
                                self.pressure[c] = sum / diag;
                            }
                        }
                    }
                }
            }
        }
        // Subtract gradient. Scale folds dt and dx so pressure units are velocity*dx.
        let _ = scale;
        for k in 0..nz {
            for j in 0..ny {
                for i in 0..nx {
                    let c = self.ci(i, j, k);
                    let pc = self.pressure[c];
                    let fluid_here = self.cell[c] == Cell::Fluid;
                    // x- face
                    if i > 0 {
                        let left = self.ci(i - 1, j, k);
                        if (fluid_here || self.cell[left] == Cell::Fluid)
                            && !self.solid[c]
                            && !self.solid[left]
                        {
                            let pl = if self.cell[left] == Cell::Fluid { self.pressure[left] } else { 0.0 };
                            let pr = if fluid_here { pc } else { 0.0 };
                            let idx = self.mac.ui(i, j, k);
                            self.mac.u[idx] -= (pr - pl) / self.prm.dx;
                        }
                    }
                    if j > 0 {
                        let down = self.ci(i, j - 1, k);
                        if (fluid_here || self.cell[down] == Cell::Fluid)
                            && !self.solid[c]
                            && !self.solid[down]
                        {
                            let pd = if self.cell[down] == Cell::Fluid { self.pressure[down] } else { 0.0 };
                            let pu = if fluid_here { pc } else { 0.0 };
                            let idx = self.mac.vi(i, j, k);
                            self.mac.v[idx] -= (pu - pd) / self.prm.dx;
                        }
                    }
                    if k > 0 {
                        let back = self.ci(i, j, k - 1);
                        if (fluid_here || self.cell[back] == Cell::Fluid)
                            && !self.solid[c]
                            && !self.solid[back]
                        {
                            let pb = if self.cell[back] == Cell::Fluid { self.pressure[back] } else { 0.0 };
                            let pf = if fluid_here { pc } else { 0.0 };
                            let idx = self.mac.wi(i, j, k);
                            self.mac.w[idx] -= (pf - pb) / self.prm.dx;
                        }
                    }
                }
            }
        }
    }

    /// Give air faces the nearest fluid-face velocity, a few sweeps, so particles leaving the
    /// fluid read a sensible value.
    fn extrapolate(&mut self) {
        for _ in 0..4 {
            sweep(&mut self.mac.u, &self.mac.wu, self.nx + 1, self.ny, self.nz);
            sweep(&mut self.mac.v, &self.mac.wv, self.nx, self.ny + 1, self.nz);
            sweep(&mut self.mac.w, &self.mac.ww, self.nx, self.ny, self.nz + 1);
        }
    }

    /// Grid back to particles, FLIP (grid change) blended with PIC (grid value).
    fn g2p(&mut self) {
        let (nx, ny, nz) = (self.nx, self.ny, self.nz);
        let flip = self.prm.flip;
        for pt in &mut self.particles {
            let pic = [
                sample(&self.mac.u, nx + 1, ny, nz, pt.p, [0.0, 0.5, 0.5]),
                sample(&self.mac.v, nx, ny + 1, nz, pt.p, [0.5, 0.0, 0.5]),
                sample(&self.mac.w, nx, ny, nz + 1, pt.p, [0.5, 0.5, 0.0]),
            ];
            let d = [
                pic[0] - sample(&self.old.u, nx + 1, ny, nz, pt.p, [0.0, 0.5, 0.5]),
                pic[1] - sample(&self.old.v, nx, ny + 1, nz, pt.p, [0.5, 0.0, 0.5]),
                pic[2] - sample(&self.old.w, nx, ny, nz + 1, pt.p, [0.5, 0.5, 0.0]),
            ];
            for a in 0..3 {
                pt.v[a] = (1.0 - flip) * pic[a] + flip * (pt.v[a] + d[a]);
            }
        }
    }

    /// Move particles RK2, keep them in the domain and out of solids.
    fn advect(&mut self, dt: f32) {
        let (nx, ny, nz) = (self.nx, self.ny, self.nz);
        let eps = 1e-3;
        let mut parts = std::mem::take(&mut self.particles);
        for pt in &mut parts {
            // Midpoint.
            let mut mid = pt.p;
            for a in 0..3 {
                mid[a] += 0.5 * dt * pt.v[a];
            }
            let gv = [
                sample(&self.mac.u, nx + 1, ny, nz, mid, [0.0, 0.5, 0.5]),
                sample(&self.mac.v, nx, ny + 1, nz, mid, [0.5, 0.0, 0.5]),
                sample(&self.mac.w, nx, ny, nz + 1, mid, [0.5, 0.5, 0.0]),
            ];
            for a in 0..3 {
                pt.p[a] += dt * gv[a];
            }
            // Clamp to the open part of the domain.
            pt.p[0] = pt.p[0].clamp(eps, nx as f32 - eps);
            pt.p[1] = pt.p[1].clamp(eps, ny as f32 - eps);
            pt.p[2] = pt.p[2].clamp(eps, nz as f32 - eps);
            self.push_out(pt);
        }
        self.particles = parts;
    }

    fn push_out(&self, pt: &mut Particle) {
        let (i, j, k) = (pt.p[0] as i32, pt.p[1] as i32, pt.p[2] as i32);
        if !self.in_grid(i, j, k) {
            return;
        }
        if self.solid[self.ci(i as usize, j as usize, k as usize)] {
            // Push to the nearest open face of this cell.
            let f = [pt.p[0] - i as f32, pt.p[1] - j as f32, pt.p[2] - k as f32];
            let dist = [f[0].min(1.0 - f[0]), f[1].min(1.0 - f[1]), f[2].min(1.0 - f[2])];
            let a = (0..3).min_by(|x, y| dist[*x].partial_cmp(&dist[*y]).unwrap()).unwrap();
            let open = |ii: i32, jj: i32, kk: i32| !self.in_grid(ii, jj, kk) || !self.solid[self.ci(ii as usize, jj as usize, kk as usize)];
            let (lo, hi) = match a {
                0 => (open(i - 1, j, k), open(i + 1, j, k)),
                1 => (open(i, j - 1, k), open(i, j + 1, k)),
                _ => (open(i, j, k - 1), open(i, j, k + 1)),
            };
            let eps = 1e-3;
            if f[a] < 0.5 && lo {
                pt.p[a] = [i, j, k][a] as f32 - eps;
            } else if hi {
                pt.p[a] = [i, j, k][a] as f32 + 1.0 + eps;
            } else if lo {
                pt.p[a] = [i, j, k][a] as f32 - eps;
            }
            pt.v[a] = 0.0;
        }
    }
}

fn splat(field: &mut [f32], wsum: &mut [f32], sx: usize, sy: usize, sz: usize, p: [f32; 3], off: [f32; 3], val: f32) {
    let g = [p[0] - off[0], p[1] - off[1], p[2] - off[2]];
    let b = [g[0].floor() as i32, g[1].floor() as i32, g[2].floor() as i32];
    let f = [g[0] - b[0] as f32, g[1] - b[1] as f32, g[2] - b[2] as f32];
    for dk in 0..2 {
        for dj in 0..2 {
            for di in 0..2 {
                let (ci, cj, ck) = (b[0] + di, b[1] + dj, b[2] + dk);
                if ci < 0 || cj < 0 || ck < 0 || ci as usize >= sx || cj as usize >= sy || ck as usize >= sz {
                    continue;
                }
                let (wx, _) = tri(f[0]);
                let (wy, _) = tri(f[1]);
                let (wz, _) = tri(f[2]);
                let w = (if di == 1 { f[0] } else { wx }) * (if dj == 1 { f[1] } else { wy }) * (if dk == 1 { f[2] } else { wz });
                let idx = ci as usize + sx * (cj as usize + sy * ck as usize);
                field[idx] += w * val;
                wsum[idx] += w;
            }
        }
    }
}

fn norm(field: &mut [f32], wsum: &[f32]) {
    for (f, w) in field.iter_mut().zip(wsum) {
        if *w > 1e-8 {
            *f /= *w;
        }
    }
}

fn sample(field: &[f32], sx: usize, sy: usize, sz: usize, p: [f32; 3], off: [f32; 3]) -> f32 {
    let g = [p[0] - off[0], p[1] - off[1], p[2] - off[2]];
    let b = [g[0].floor() as i32, g[1].floor() as i32, g[2].floor() as i32];
    let f = [g[0] - b[0] as f32, g[1] - b[1] as f32, g[2] - b[2] as f32];
    let mut val = 0.0;
    for dk in 0..2 {
        for dj in 0..2 {
            for di in 0..2 {
                let (ci, cj, ck) = (b[0] + di, b[1] + dj, b[2] + dk);
                let ci = ci.clamp(0, sx as i32 - 1) as usize;
                let cj = cj.clamp(0, sy as i32 - 1) as usize;
                let ck = ck.clamp(0, sz as i32 - 1) as usize;
                let w = (if di == 1 { f[0] } else { 1.0 - f[0] }) * (if dj == 1 { f[1] } else { 1.0 - f[1] }) * (if dk == 1 { f[2] } else { 1.0 - f[2] });
                val += w * field[ci + sx * (cj + sy * ck)];
            }
        }
    }
    val
}

/// One extrapolation sweep: a zero-weight face takes the average of its set neighbours.
fn sweep(field: &mut [f32], wsum: &[f32], sx: usize, sy: usize, sz: usize) {
    let snapshot = field.to_vec();
    for k in 0..sz {
        for j in 0..sy {
            for i in 0..sx {
                let idx = i + sx * (j + sy * k);
                if wsum[idx] > 1e-8 {
                    continue;
                }
                let mut sum = 0.0;
                let mut n = 0.0;
                for (di, dj, dk) in [(-1i32, 0i32, 0i32), (1, 0, 0), (0, -1, 0), (0, 1, 0), (0, 0, -1), (0, 0, 1)] {
                    let (ni, nj, nk) = (i as i32 + di, j as i32 + dj, k as i32 + dk);
                    if ni < 0 || nj < 0 || nk < 0 || ni as usize >= sx || nj as usize >= sy || nk as usize >= sz {
                        continue;
                    }
                    let nidx = ni as usize + sx * (nj as usize + sy * nk as usize);
                    if wsum[nidx] > 1e-8 {
                        sum += snapshot[nidx];
                        n += 1.0;
                    }
                }
                if n > 0.0 {
                    field[idx] = sum / n;
                }
            }
        }
    }
}
