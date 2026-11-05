use crate::{Cell, Fluid, Params, Particle};

fn basin(nx: usize, ny: usize, nz: usize) -> Fluid {
    let mut f = Fluid::new(nx, ny, nz, Params { iters: 60, ..Default::default() });
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                // Floor and the four side walls are solid; top is open air.
                if j == 0 || i == 0 || i == nx - 1 || k == 0 || k == nz - 1 {
                    f.solid[i + nx * (j + ny * k)] = true;
                }
            }
        }
    }
    f
}

fn run(f: &mut Fluid, steps: usize, dt: f32) {
    for _ in 0..steps {
        f.step(dt);
    }
}

#[test]
fn particle_count_is_conserved_and_stays_in_the_basin() {
    let mut f = basin(10, 16, 10);
    let mut seed = 1;
    f.fill_box([1, 6, 1], [9, 12, 9], 8, &mut seed);
    let start = f.particles.len();
    run(&mut f, 200, 1.0 / 60.0);
    assert_eq!(f.particles.len(), start);
    for p in &f.particles {
        assert!(p.p[0] > 0.0 && p.p[0] < 10.0, "x {:?}", p.p);
        assert!(p.p[1] > 0.0 && p.p[1] < 16.0, "y {:?}", p.p);
        assert!(p.p[2] > 0.0 && p.p[2] < 10.0, "z {:?}", p.p);
    }
}

#[test]
fn dropped_water_settles_to_a_flat_level_and_stays() {
    let mut f = basin(12, 20, 12);
    let mut seed = 7;
    // A tall thin column dropped into the basin.
    f.fill_box([2, 4, 2], [6, 16, 6], 8, &mut seed);
    let n = f.particles.len() as f32;
    run(&mut f, 600, 1.0 / 60.0);
    // Surface height per interior column: the top particle in it.
    let (nx, nz) = (12, 12);
    let mut top = vec![0.0f32; nx * nz];
    for p in &f.particles {
        let (i, k) = (p.p[0] as usize, p.p[2] as usize);
        let idx = i + nx * k;
        top[idx] = top[idx].max(p.p[1]);
    }
    let covered: Vec<f32> = (1..nx - 1)
        .flat_map(|i| (1..nz - 1).map(move |k| (i, k)))
        .map(|(i, k)| top[i + nx * k])
        .filter(|h| *h > 0.5)
        .collect();
    let lo = covered.iter().cloned().fold(f32::MAX, f32::min);
    let hi = covered.iter().cloned().fold(0.0f32, f32::max);
    // Flat to about a cell.
    assert!(hi - lo <= 2.0, "surface not flat: {lo}..{hi}");
    // Water spread over the floor (volume preserved, not a thin spike).
    assert!(covered.len() as f32 >= n / (8.0 * 4.0), "did not spread: {} cols", covered.len());
    // Nothing leaked.
    assert_eq!(f.particles.len(), n as usize);
}

#[test]
fn water_runs_downhill_off_a_step() {
    let mut f = basin(20, 12, 6);
    // A solid shelf on the left half, up to y=5.
    for k in 0..6 {
        for j in 0..=5 {
            for i in 0..10 {
                f.solid[i + 20 * (j + 12 * k)] = true;
            }
        }
    }
    let mut seed = 3;
    f.fill_box([2, 6, 1], [8, 10, 5], 8, &mut seed);
    run(&mut f, 400, 1.0 / 60.0);
    // Most water ended on the low (right) side.
    let right = f.particles.iter().filter(|p| p.p[0] > 10.0).count();
    assert!(right as f32 > 0.5 * f.particles.len() as f32, "water did not flow down: {right}/{}", f.particles.len());
}

#[test]
fn still_water_barely_moves() {
    let mut f = basin(10, 14, 10);
    let mut seed = 5;
    f.fill_box([1, 1, 1], [9, 6, 9], 8, &mut seed);
    run(&mut f, 300, 1.0 / 60.0); // settle
    let before: Vec<[f32; 3]> = f.particles.iter().map(|p| p.p).collect();
    run(&mut f, 60, 1.0 / 60.0);
    let moved: f32 = f
        .particles
        .iter()
        .zip(&before)
        .map(|(p, b)| {
            let d = [p.p[0] - b[0], p.p[1] - b[1], p.p[2] - b[2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        })
        .sum::<f32>()
        / f.particles.len() as f32;
    assert!(moved < 0.15, "settled water drifts {moved} per particle per second");
}

#[test]
fn a_cell_never_holds_a_wild_number_after_settling() {
    let mut f = basin(10, 16, 10);
    let mut seed = 9;
    f.fill_box([1, 1, 1], [9, 8, 9], 8, &mut seed);
    run(&mut f, 400, 1.0 / 60.0);
    let (nx, ny, nz) = (10, 16, 10);
    let mut count = vec![0u32; nx * ny * nz];
    for p in &f.particles {
        let (i, j, k) = (p.p[0] as usize, p.p[1] as usize, p.p[2] as usize);
        count[i + nx * (j + ny * k)] += 1;
    }
    let max = *count.iter().max().unwrap();
    // Incompressible: no cell should pile far past the fill density of 8.
    assert!(max <= 20, "a cell holds {max} particles (clumping)");
}
