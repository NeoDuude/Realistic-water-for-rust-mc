struct Params {
  dims : vec4<i32>,
  sim  : vec4<f32>,
  misc : vec4<f32>,
};
@group(0) @binding(0) var<uniform> P : Params;
fn nx() -> i32 { return P.dims.x; }
fn ny() -> i32 { return P.dims.y; }
fn nz() -> i32 { return P.dims.z; }
fn dx() -> f32 { return P.sim.x; }
fn ci(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
fn inGrid(i:i32,j:i32,k:i32)->bool { return i>=0 && j>=0 && k>=0 && i<nx() && j<ny() && k<nz(); }
fn ui(i:i32,j:i32,k:i32)->i32 { return i + (nx()+1)*(j + ny()*k); }
fn vi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + (ny()+1)*k); }
fn wi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
const AIR:u32=0u; const FLUID:u32=1u; const SOLID:u32=2u;
@group(0) @binding(1) var<storage, read> pres : array<f32>;
@group(0) @binding(2) var<storage, read> cell : array<u32>;
@group(0) @binding(3) var<storage, read> solid : array<u32>;
@group(0) @binding(4) var<storage, read_write> u : array<f32>;
@group(0) @binding(5) var<storage, read_write> v : array<f32>;
@group(0) @binding(6) var<storage, read_write> w : array<f32>;
fn pOf(i:i32,j:i32,k:i32)->f32 { if (!inGrid(i,j,k)) { return 0.0; } if (cell[ci(i,j,k)]==FLUID) { return pres[ci(i,j,k)]; } return 0.0; }
fn sOf(i:i32,j:i32,k:i32)->bool { if (!inGrid(i,j,k)) { return true; } return solid[ci(i,j,k)]!=0u; }
fn fluidOf(i:i32,j:i32,k:i32)->bool { if (!inGrid(i,j,k)) { return false; } return cell[ci(i,j,k)]==FLUID; }
@compute @workgroup_size(4,4,4)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let i=i32(gid.x); let j=i32(gid.y); let k=i32(gid.z);
  if (!inGrid(i,j,k)) { return; }
  if (i>0) { if ((fluidOf(i,j,k)||fluidOf(i-1,j,k)) && !sOf(i,j,k) && !sOf(i-1,j,k)) { u[ui(i,j,k)] = u[ui(i,j,k)] - (pOf(i,j,k)-pOf(i-1,j,k))/dx(); } }
  if (j>0) { if ((fluidOf(i,j,k)||fluidOf(i,j-1,k)) && !sOf(i,j,k) && !sOf(i,j-1,k)) { v[vi(i,j,k)] = v[vi(i,j,k)] - (pOf(i,j,k)-pOf(i,j-1,k))/dx(); } }
  if (k>0) { if ((fluidOf(i,j,k)||fluidOf(i,j,k-1)) && !sOf(i,j,k) && !sOf(i,j,k-1)) { w[wi(i,j,k)] = w[wi(i,j,k)] - (pOf(i,j,k)-pOf(i,j,k-1))/dx(); } }
}
