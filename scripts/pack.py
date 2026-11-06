#!/usr/bin/env python3
"""Package Real Physics mod in all three formats the Horizon launcher accepts:
  - .mrpack  (Modrinth modpack format)
  - .zip     (shader/resource pack)
  - .hshader (Horizon shader pack — our best guess at the structure)
The user can try each one to see which the launcher imports."""
import json, os, re, zipfile, hashlib, shutil

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUNDLE = os.path.join(ROOT, "bundle")
SHADERS = os.path.join(ROOT, "shaders")
DIST = os.path.join(ROOT, "dist")

def version():
    toml_path = os.path.join(BUNDLE, "horizon-mixin.toml")
    if os.path.exists(toml_path):
        toml = open(toml_path).read()
        m = re.search(r'(?m)^version\s*=\s*"([^"]+)"', toml)
        if m:
            return m.group(1)
    return "0.1.0"

def collect_shaders():
    """Collect all .wgsl shader files."""
    files = []
    for f in sorted(os.listdir(SHADERS)):
        if f.endswith(".wgsl"):
            files.append(os.path.join(SHADERS, f))
    return files

def collect_bundle():
    """Collect all bundle source files (engine + helpers)."""
    files = []
    for d, _, fs in os.walk(BUNDLE):
        if "target" in d.split(os.sep):
            continue
        for f in sorted(fs):
            if f == "Cargo.lock":
                continue
            files.append(os.path.join(d, f))
    return files

def sha512(path):
    h = hashlib.sha512()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()

def build_mrpack(ver):
    """Build a Modrinth .mrpack modpack file."""
    out = os.path.join(DIST, f"realphysics-{ver}.mrpack")
    index = {
        "formatVersion": 1,
        "game": "minecraft",
        "versionId": f"realphysics-{ver}",
        "name": "Real Physics",
        "summary": "Realistic water, falling trees, sliding sand — GPU fluid sim with WGSL shaders",
        "files": [],
        "dependencies": {
            "minecraft": "26.3"
        }
    }
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        # Shaders go in overrides
        for sp in collect_shaders():
            arcname = "overrides/shaderpacks/realphysics/shaders/" + os.path.basename(sp)
            z.write(sp, arcname)
        # Bundle (native mod source) in overrides
        for bp in collect_bundle():
            rel = os.path.relpath(bp, BUNDLE)
            arcname = "overrides/mods/realphysics/" + rel
            z.write(bp, arcname)
        # pack.mcmeta in overrides root
        pack_meta = {
            "pack": {
                "pack_format": 34,
                "description": "Real Physics — realistic water, sand, trees"
            }
        }
        z.writestr("overrides/pack.mcmeta", json.dumps(pack_meta, indent=2))
        # The modrinth index
        z.writestr("modrinth.index.json", json.dumps(index, indent=2))
    print(f"  mrpack: {out}")
    return out

def build_zip(ver):
    """Build a plain .zip shader/resource pack."""
    out = os.path.join(DIST, f"realphysics-{ver}.zip")
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        # pack.mcmeta at root
        pack_meta = {
            "pack": {
                "pack_format": 34,
                "description": "Real Physics — realistic water, sand, trees"
            }
        }
        z.writestr("pack.mcmeta", json.dumps(pack_meta, indent=2))
        # Shaders
        for sp in collect_shaders():
            arcname = "assets/realphysics/shaders/" + os.path.basename(sp)
            z.write(sp, arcname)
        # Bundle source
        for bp in collect_bundle():
            rel = os.path.relpath(bp, BUNDLE)
            arcname = "mods/realphysics/" + rel
            z.write(bp, arcname)
    print(f"  zip:    {out}")
    return out

def build_hshader(ver):
    """Build a .hshader — likely a zip of shaders with a manifest."""
    out = os.path.join(DIST, f"realphysics-{ver}.hshader")
    manifest = {
        "id": "realphysics",
        "name": "Real Physics",
        "version": ver,
        "description": "Realistic water, falling trees, sliding sand — GPU fluid sim",
        "game_version": "26.3",
        "shaders": []
    }
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        for sp in collect_shaders():
            fname = os.path.basename(sp)
            arcname = "shaders/" + fname
            z.write(sp, arcname)
            # Classify shader type from filename
            if ".comp." in fname:
                stype = "compute"
            elif ".vert." in fname:
                stype = "vertex"
            elif ".frag." in fname:
                stype = "fragment"
            else:
                stype = "unknown"
            manifest["shaders"].append({
                "path": arcname,
                "type": stype,
                "format": "wgsl"
            })
        # Bundle source alongside
        for bp in collect_bundle():
            rel = os.path.relpath(bp, BUNDLE)
            arcname = "native/" + rel
            z.write(bp, arcname)
        z.writestr("shader.json", json.dumps(manifest, indent=2))
    print(f"  hshader: {out}")
    return out

def main():
    ver = version()
    os.makedirs(DIST, exist_ok=True)
    # Clean old builds
    for f in os.listdir(DIST):
        os.remove(os.path.join(DIST, f))
    print(f"Building Real Physics v{ver}:")
    build_mrpack(ver)
    build_zip(ver)
    build_hshader(ver)
    print("Done. Try each file in the Horizon launcher.")

if __name__ == "__main__":
    main()
