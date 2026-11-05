use naga::valid::{Capabilities, ValidationFlags, Validator};
fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "../shaders".into());
    let mut fail = 0;
    let mut n = 0;
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap()
        .filter_map(|e| e.ok()).map(|e| e.path())
        .filter(|p| p.extension().map(|x| x=="wgsl").unwrap_or(false)).collect();
    files.sort();
    for p in files {
        n += 1;
        let src = std::fs::read_to_string(&p).unwrap();
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        match naga::front::wgsl::parse_str(&src) {
            Err(e) => { fail += 1; println!("PARSE FAIL {name}:\n{}", e.emit_to_string(&src)); }
            Ok(module) => {
                let mut v = Validator::new(ValidationFlags::all(), Capabilities::all());
                match v.validate(&module) {
                    Ok(_) => println!("ok   {name}"),
                    Err(e) => { fail += 1; println!("VALIDATE FAIL {name}: {}", e.emit_to_string(&src)); }
                }
            }
        }
    }
    println!("--- {n} shaders, {fail} failed");
    if fail > 0 { std::process::exit(1); }
}
