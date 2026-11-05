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
struct Particle { pos : vec4<f32>, vel : vec4<f32>, };
// accum*: interleaved [value*FP, weight*FP] per face.
@group(0) @binding(1) var<storage, read> parts : array<Particle>;
@group(0) @binding(2) var<storage, read_write> accU : array<atomic<i32>>;
@group(0) @binding(3) var<storage, read_write> accV : array<atomic<i32>>;
@group(0) @binding(4) var<storage, read_write> accW : array<atomic<i32>>;

fn splat(comp:i32, x:vec3<f32>, val:f32) {
  var off = vec3<f32>(0.5,0.5,0.5);
  var sx = nx(); var sy = ny(); var sz = nz();
  if (comp==0) { off.x=0.0; sx=nx()+1; }
  if (comp==1) { off.y=0.0; sy=ny()+1; }
  if (comp==2) { off.z=0.0; sz=nz()+1; }
  let g = x - off;
  let b = vec3<i32>(floor(g));
  let f = g - floor(g);
  for (var dk=0; dk<2; dk++) {
    for (var dj=0; dj<2; dj++) {
      for (var di=0; di<2; di++) {
        let c = b + vec3<i32>(di,dj,dk);
        if (c.x<0||c.y<0||c.z<0||c.x>=sx||c.y>=sy||c.z>=sz) { continue; }
        let wx = select(1.0-f.x, f.x, di==1);
        let wy = select(1.0-f.y, f.y, dj==1);
        let wz = select(1.0-f.z, f.z, dk==1);
        let w = wx*wy*wz;
        let idx = c.x + sx*(c.y + sy*c.z);
        let vi2 = 2*idx;
        if (comp==0) { atomicAdd(&accU[vi2], i32(round(w*val*FP))); atomicAdd(&accU[vi2+1], i32(round(w*FP))); }
        else if (comp==1) { atomicAdd(&accV[vi2], i32(round(w*val*FP))); atomicAdd(&accV[vi2+1], i32(round(w*FP))); }
        else { atomicAdd(&accW[vi2], i32(round(w*val*FP))); atomicAdd(&accW[vi2+1], i32(round(w*FP))); }
      }
    }
  }
}
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let i = i32(gid.x);
  if (i >= P.dims.w) { return; }
  let p = parts[i];
  if (p.pos.w < 0.0) { return; }
  splat(0, p.pos.xyz, p.vel.x);
  splat(1, p.pos.xyz, p.vel.y);
  splat(2, p.pos.xyz, p.vel.z);
}
