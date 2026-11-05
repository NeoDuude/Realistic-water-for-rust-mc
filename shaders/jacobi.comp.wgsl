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
@group(0) @binding(1) var<storage, read> pin : array<f32>;
@group(0) @binding(2) var<storage, read_write> pout : array<f32>;
@group(0) @binding(3) var<storage, read> divg : array<f32>;
@group(0) @binding(4) var<storage, read> cell : array<u32>;
@group(0) @binding(5) var<storage, read> solid : array<u32>;
fn isSolid(i:i32,j:i32,k:i32)->bool { if (!inGrid(i,j,k)) { return true; } return solid[ci(i,j,k)]!=0u; }
@compute @workgroup_size(4,4,4)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let i=i32(gid.x); let j=i32(gid.y); let k=i32(gid.z);
  if (!inGrid(i,j,k)) { return; }
  let c = ci(i,j,k);
  if (cell[c] != FLUID) { pout[c] = 0.0; return; }
  var diag = 0.0; var sum = divg[c];
  var offs = array<vec3<i32>,6>(vec3<i32>(-1,0,0),vec3<i32>(1,0,0),vec3<i32>(0,-1,0),vec3<i32>(0,1,0),vec3<i32>(0,0,-1),vec3<i32>(0,0,1));
  for (var t=0; t<6; t++) {
    let o = offs[t];
    let ni=i+o.x; let nj=j+o.y; let nk=k+o.z;
    if (isSolid(ni,nj,nk)) { continue; }
    diag = diag + 1.0;
    if (inGrid(ni,nj,nk)) { if (cell[ci(ni,nj,nk)]==FLUID) { sum = sum + pin[ci(ni,nj,nk)]; } }
  }
  if (diag > 0.0) { pout[c] = sum / diag; } else { pout[c] = 0.0; }
}
