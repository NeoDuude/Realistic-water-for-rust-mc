struct Params {
  dims : vec4<i32>,
  sim  : vec4<f32>,
  misc : vec4<f32>,
};
@group(0) @binding(0) var<uniform> P : Params;
fn nx() -> i32 { return P.dims.x; }
fn ny() -> i32 { return P.dims.y; }
fn nz() -> i32 { return P.dims.z; }
fn ci(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }
fn inGrid(i:i32,j:i32,k:i32)->bool { return i>=0 && j>=0 && k>=0 && i<nx() && j<ny() && k<nz(); }
@group(0) @binding(1) var<storage, read> fin : array<f32>;
@group(0) @binding(2) var<storage, read_write> fout : array<f32>;
@group(0) @binding(3) var<storage, read> acc : array<i32>;
// misc: (targetPerCell, sx, sy, sz) for this face field.
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let idx = i32(gid.x);
  let sx = i32(P.misc.y); let sy = i32(P.misc.z); let sz = i32(P.misc.w);
  if (idx >= sx*sy*sz) { return; }
  fout[idx] = fin[idx];
  if (acc[2*idx+1] > 0) { return; }
  let i = idx % sx; let j = (idx/sx) % sy; let k = idx/(sx*sy);
  var sum = 0.0; var n = 0.0;
  var offs = array<vec3<i32>,6>(vec3<i32>(-1,0,0),vec3<i32>(1,0,0),vec3<i32>(0,-1,0),vec3<i32>(0,1,0),vec3<i32>(0,0,-1),vec3<i32>(0,0,1));
  for (var t=0; t<6; t++) {
    let o = offs[t]; let ni=i+o.x; let nj=j+o.y; let nk=k+o.z;
    if (ni<0||nj<0||nk<0||ni>=sx||nj>=sy||nk>=sz) { continue; }
    let nidx = ni + sx*(nj + sy*nk);
    if (acc[2*nidx+1] > 0) { sum = sum + fin[nidx]; n = n + 1.0; }
  }
  if (n > 0.0) { fout[idx] = sum / n; }
}
