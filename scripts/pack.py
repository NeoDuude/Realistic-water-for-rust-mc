#!/usr/bin/env python3
"""Build the two release files, in Horizon's real formats:

  dist/real_water-<ver>.hshader   the water look (shader pack, horizon-shaders.toml + WGSL)
                                  -> drag into the launcher, or put in shaderpacks/
  dist/realphysics-<ver>.hnmod    trees, sand and block water (native mod)
                                  -> launcher Mods page > Add from GitHub

Layouts follow horizon-shader-api (package.rs) and horizon-mods (index/native.rs).
"""
import json, os, re, zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUNDLE = os.path.join(ROOT, "bundle")
PACK = os.path.join(ROOT, "shaderpack")
DIST = os.path.join(ROOT, "dist")

# horizon-mixins commit the launcher builds native mods with (launcher native_mods::MIXINS_REV).
MIXINS_REV = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
# NATIVE_HOOKS_API of the Horizon this mod was tested on; it uses only hook points,
# so it builds on that Horizon or any later one.
HOOKS_API = 411
FIXED = (1980, 1, 1, 0, 0, 0)


def field(text, key):
    m = re.search(r'(?m)^%s\s*=\s*"([^"]+)"' % key, text)
    if not m:
        raise SystemExit(f"missing {key}")
    return m.group(1)


def add(z, path, arc):
    info = zipfile.ZipInfo(arc, FIXED)
    info.compress_type = zipfile.ZIP_DEFLATED
    info.external_attr = 0o100644 << 16
    with open(path, "rb") as f:
        z.writestr(info, f.read())


def walk(top):
    for d, dirs, fs in os.walk(top):
        dirs[:] = sorted(x for x in dirs if not x.startswith(".") and x != "target")
        for f in sorted(fs):
            if not f.startswith(".") and f != "Cargo.lock":
                p = os.path.join(d, f)
                yield p, os.path.relpath(p, top).replace(os.sep, "/")


def build_hshader():
    toml = open(os.path.join(PACK, "horizon-shaders.toml")).read()
    pid, ver = field(toml, "id"), field(toml, "version")
    out = os.path.join(DIST, f"{pid}-{ver}.hshader")
    with zipfile.ZipFile(out, "w") as z:
        for p, rel in walk(PACK):
            add(z, p, rel)
    return out


def build_hnmod():
    toml = open(os.path.join(BUNDLE, "horizon-mixin.toml")).read()
    mid, ver = field(toml, "id"), field(toml, "version")
    manifest = {
        "schema": 1,
        "id": mid,
        "version": ver,
        "game": "26.3",
        "horizon": field(toml, "horizon_revision"),
        "mixins": MIXINS_REV,
        "hooks": HOOKS_API,
    }
    out = os.path.join(DIST, f"{mid}-{ver}.hnmod")
    with zipfile.ZipFile(out, "w") as z:
        info = zipfile.ZipInfo("horizon-native.json", FIXED)
        info.compress_type = zipfile.ZIP_DEFLATED
        info.external_attr = 0o100644 << 16
        z.writestr(info, json.dumps(manifest, indent=2) + "\n")
        for p, rel in walk(BUNDLE):
            add(z, p, f"mixins/{mid}/{rel}")
    return out


def main():
    os.makedirs(DIST, exist_ok=True)
    for f in os.listdir(DIST):
        os.remove(os.path.join(DIST, f))
    print(build_hshader())
    print(build_hnmod())


if __name__ == "__main__":
    main()
