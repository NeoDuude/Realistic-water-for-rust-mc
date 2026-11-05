struct Params {
  dims : vec4<i32>,
  sim  : vec4<f32>,
  misc : vec4<f32>,
};
@group(0) @binding(0) var<uniform> P : Params;
const FP : f32 = 65536.0;
fn nx() -> i32 { return P.dims.x; }
fn ny() -> i32 { return P.dims.y; }
fn nz() -> i32 { return P.dims.z; }
fn dx() -> f32 { return P.sim.x; }
fn ci(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
fn inGrid(i:i32,j:i32,k:i32)->bool { return i>=0 && j>=0 && k>=0 && i<nx() && j<ny() && k<nz(); }
fn ui(i:i32,j:i32,k:i32)->i32 { return i + (nx()+1)*(j + ny()*k); }
fn vi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + (ny()+1)*k); }
fn wi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
@group(0) @binding(1) var<storage, read> solid : array<u32>;
@group(0) @binding(2) var<storage, read_write> u : array<f32>;
@group(0) @binding(3) var<storage, read_write> v : array<f32>;
@group(0) @binding(4) var<storage, read_write> w : array<f32>;
fn isSolid(i:i32,j:i32,k:i32)->bool { if (!inGrid(i,j,k)) { return true; } return solid[ci(i,j,k)]!=0u; }
@compute @workgroup_size(4,4,4)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let i = i32(gid.x); let j = i32(gid.y); let k = i32(gid.z);
  // u faces: 0..=nx
  if (i<=nx() && j<ny() && k<nz()) { if (isSolid(i-1,j,k)||isSolid(i,j,k)) { u[ui(i,j,k)] = 0.0; } }
  if (i<nx() && j<=ny() && k<nz()) { if (isSolid(i,j-1,k)||isSolid(i,j,k)) { v[vi(i,j,k)] = 0.0; } }
  if (i<nx() && j<ny() && k<=nz()) { if (isSolid(i,j,k-1)||isSolid(i,j,k)) { w[wi(i,j,k)] = 0.0; } }
}
