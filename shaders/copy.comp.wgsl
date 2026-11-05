struct Params {
  dims : vec4<i32>,  // nx, ny, nz, particleCount
  sim  : vec4<f32>,  // dx, gravity, flip, dt
  misc : vec4<f32>,  // targetPerCell, _, _, _
};
@group(0) @binding(0) var<uniform> P : Params;

const FP : f32 = 65536.0;

fn nx() -> i32 { return P.dims.x; }
fn ny() -> i32 { return P.dims.y; }
fn nz() -> i32 { return P.dims.z; }
fn ci(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
fn inGrid(i:i32,j:i32,k:i32)->bool { return i>=0 && j>=0 && k>=0 && i<nx() && j<ny() && k<nz(); }
// face index helpers (sizes: u=(nx+1,ny,nz) v=(nx,ny+1,nz) w=(nx,ny,nz+1))
fn ui(i:i32,j:i32,k:i32)->i32 { return i + (nx()+1)*(j + ny()*k); }
fn vi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + (ny()+1)*k); }
fn wi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
@group(0) @binding(1) var<storage, read> src : array<f32>;
@group(0) @binding(2) var<storage, read_write> dst : array<f32>;
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let i = i32(gid.x);
  if (i >= i32(arrayLength(&src))) { return; }
  dst[i] = src[i];
}
