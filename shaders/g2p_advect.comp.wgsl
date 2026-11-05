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
struct Particle { pos : vec4<f32>, vel : vec4<f32>, };
@group(0) @binding(1) var<storage, read_write> parts : array<Particle>;
@group(0) @binding(2) var<storage, read> u : array<f32>;
@group(0) @binding(3) var<storage, read> v : array<f32>;
@group(0) @binding(4) var<storage, read> w : array<f32>;
@group(0) @binding(5) var<storage, read> uo : array<f32>;
@group(0) @binding(6) var<storage, read> vo : array<f32>;
@group(0) @binding(7) var<storage, read> wo : array<f32>;
@group(0) @binding(8) var<storage, read> solid : array<u32>;

fn ui(i:i32,j:i32,k:i32)->i32 { return i + (nx()+1)*(j + ny()*k); }
fn vi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + (ny()+1)*k); }
fn wi(i:i32,j:i32,k:i32)->i32 { return i + nx()*(j + ny()*k); }

fn sampleField(comp:i32, useOld:bool, x:vec3<f32>) -> f32 {
  var off = vec3<f32>(0.5,0.5,0.5);
  var sx=nx(); var sy=ny(); var sz=nz();
  if (comp==0) { off.x=0.0; sx=nx()+1; }
  if (comp==1) { off.y=0.0; sy=ny()+1; }
  if (comp==2) { off.z=0.0; sz=nz()+1; }
  let g = x - off;
  let b = vec3<i32>(floor(g));
  let f = g - floor(g);
  var val = 0.0;
  for (var dk=0; dk<2; dk++) {
    for (var dj=0; dj<2; dj++) {
      for (var di=0; di<2; di++) {
        var c = b + vec3<i32>(di,dj,dk);
        c = clamp(c, vec3<i32>(0,0,0), vec3<i32>(sx-1,sy-1,sz-1));
        let wx = select(1.0-f.x, f.x, di==1);
        let wy = select(1.0-f.y, f.y, dj==1);
        let wz = select(1.0-f.z, f.z, dk==1);
        let wt = wx*wy*wz;
        let idx = c.x + sx*(c.y + sy*c.z);
        var s = 0.0;
        if (comp==0) { s = select(u[idx], uo[idx], useOld); }
        else if (comp==1) { s = select(v[idx], vo[idx], useOld); }
        else { s = select(w[idx], wo[idx], useOld); }
        val = val + wt*s;
      }
    }
  }
  return val;
}
fn gridVel(x:vec3<f32>, useOld:bool)->vec3<f32> {
  return vec3<f32>(sampleField(0,useOld,x), sampleField(1,useOld,x), sampleField(2,useOld,x));
}
fn isSolid(i:i32,j:i32,k:i32)->bool { if (!inGrid(i,j,k)) { return false; } return solid[ci(i,j,k)]!=0u; }

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid : vec3<u32>) {
  let n = i32(gid.x);
  if (n >= P.dims.w) { return; }
  var p = parts[n];
  if (p.pos.w < 0.0) { return; }
  let flip = P.sim.z; let dt = P.sim.w;
  let pic = gridVel(p.pos.xyz, false);
  let old = gridVel(p.pos.xyz, true);
  let d = pic - old;
  p.vel = vec4<f32>((1.0-flip)*pic + flip*(p.vel.xyz + d), 0.0);
  // RK2
  let mid = p.pos.xyz + 0.5*dt*p.vel.xyz;
  let gv = gridVel(mid, false);
  var np = p.pos.xyz + dt*gv;
  let eps = 1e-3;
  np = clamp(np, vec3<f32>(eps,eps,eps), vec3<f32>(f32(nx())-eps, f32(ny())-eps, f32(nz())-eps));
  // push out of solid: nudge to nearest open face along the shallowest axis
  let ci2 = vec3<i32>(floor(np));
  if (inGrid(ci2.x,ci2.y,ci2.z) && isSolid(ci2.x,ci2.y,ci2.z)) {
    let fr = np - floor(np);
    let dist = vec3<f32>(min(fr.x,1.0-fr.x), min(fr.y,1.0-fr.y), min(fr.z,1.0-fr.z));
    var a = 0;
    if (dist.y < dist.x && dist.y <= dist.z) { a = 1; }
    if (dist.z < dist.x && dist.z < dist.y) { a = 2; }
    let base = floor(np);
    if (a==0) { if (fr.x<0.5 && !isSolid(ci2.x-1,ci2.y,ci2.z)) { np.x = base.x - eps; } else { np.x = base.x + 1.0 + eps; } p.vel.x = 0.0; }
    else if (a==1) { if (fr.y<0.5 && !isSolid(ci2.x,ci2.y-1,ci2.z)) { np.y = base.y - eps; } else { np.y = base.y + 1.0 + eps; } p.vel.y = 0.0; }
    else { if (fr.z<0.5 && !isSolid(ci2.x,ci2.y,ci2.z-1)) { np.z = base.z - eps; } else { np.z = base.z + 1.0 + eps; } p.vel.z = 0.0; }
  }
  p.pos = vec4<f32>(np, p.pos.w);
  parts[n] = p;
}
