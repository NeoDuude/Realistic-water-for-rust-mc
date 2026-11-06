#!/usr/bin/env python3
"""Package the native mod in bundle/ into dist/realphysics-<version>.hnmod.
A .hnmod is a plain zip: a manifest plus the mod's source, which the Horizon
launcher compiles against your installed Horizon at install time."""
import json, os, re, zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUNDLE = os.path.join(ROOT, "bundle")

def version():
    toml = open(os.path.join(BUNDLE, "horizon-mixin.toml")).read()
    m = re.search(r'(?m)^version\s*=\s*"([^"]+)"', toml)
    return m.group(1) if m else "0.0.0"

def horizon_rev():
    toml = open(os.path.join(BUNDLE, "horizon-mixin.toml")).read()
    m = re.search(r'horizon_revision\s*=\s*"([^"]+)"', toml)
    return m.group(1) if m else ""

def main():
    ver = version()
    os.makedirs(os.path.join(ROOT, "dist"), exist_ok=True)
    out = os.path.join(ROOT, "dist", f"realphysics-{ver}.hnmod")
    manifest = {
        "schema": 1,
        "id": "realphysics",
        "version": ver,
        "game": "26.3",
        "horizon": horizon_rev(),
    }
    with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("horizon-native.json", json.dumps(manifest, indent=2))
        for d, _, fs in os.walk(BUNDLE):
            if "target" in d.split(os.sep):
                continue
            for f in sorted(fs):
                if f == "Cargo.lock":
                    continue
                p = os.path.join(d, f)
                z.write(p, "mixins/realphysics/" + os.path.relpath(p, BUNDLE))
    print(out)

if __name__ == "__main__":
    main()
