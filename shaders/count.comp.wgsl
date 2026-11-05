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
struct Particle { pos : vec4<f32>, vel : vec4<f32>, };
@group(0) @binding(1) var<storage, read> parts : array<Particle>;
@group(0) @binding(2) var<storage, read_write> count : array<atomic<i32>>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let n = i32(gid.x);
  if (n >= P.dims.w) { return; }
  let p = parts[n];
  if (p.pos.w < 0.0) { return; }
  let i = i32(floor(p.pos.x)); let j = i32(floor(p.pos.y)); let k = i32(floor(p.pos.z));
  if (inGrid(i,j,k)) { atomicAdd(&count[ci(i,j,k)], 1); }
}
