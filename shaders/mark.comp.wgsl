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
const AIR:u32=0u; const FLUID:u32=1u; const SOLID:u32=2u;
@group(0) @binding(1) var<storage, read> solid : array<u32>;
@group(0) @binding(2) var<storage, read> count : array<i32>;
@group(0) @binding(3) var<storage, read_write> cell : array<u32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let i = i32(gid.x);
  if (i >= i32(arrayLength(&cell))) { return; }
  if (solid[i] != 0u) { cell[i] = SOLID; }
  else if (count[i] > 0) { cell[i] = FLUID; }
  else { cell[i] = AIR; }
}
