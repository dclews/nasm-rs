use std::env;
use std::process::Command;
use std::process::Stdio;
use std::path::Path;

pub enum BuildType {
    STATIC,
    SHARED,
}

fn format_x86(os: &str) -> &'static str {
    println!("OS is: {}", os);
    match os {
        "linux" => "-felf32",
        "darwin" => "-fmachos32",
        "windows" => "-fwin32",
        "none" => "-felf32",
         _ => {
            println!("OS not specified. Falling back to a.out format. If you aren't using an OS specify it as \"none\" in the target triple");
            ""
        },
    }
}

fn format_x86_64(os: &str) -> &'static str {
    println!("OS is: {}", os);
    match os {
        "linux" => "-felf64",
        "darwin" => "-felf64",
        "windows" => "-fwin64",
        "none" => "-felf64",
        _ => {
            println!("OS not specified. Falling back to a.out format. If you aren't using an OS specify it as \"none\" in the target triple");
            ""
        },
    }
}

fn parse_triple(trip: &str) -> &'static str {
    println!("Triple is {}", trip);

    let parts: Vec<&str> = match trip.rfind("/") {
        Some(i) => {
            let parts = trip[(i+1)..].split("-").collect();
            println!("Stripping base path from TARGET and treating it as target triple {:?}", parts);
            parts
        }
        None => trip.split("-").collect(),
    };

    // ARCH-VENDOR-OS-ENVIRONMENT
    // or ARCH-VENDOR-OS
    // we don't care about environ so doesn't matter if triple doesn't have it.
    let os = match parts.len() < 3 {
        true => "none",
        false => parts[2],
    };

    match parts[0] {
        "x86_64" => format_x86_64(os),
        "x86" => format_x86(os),
        _ => ""
    }
}

/// # Example
///
/// ```no_run
/// nasm::compile_library("libfoo.a", &["foo.s", "bar.s"]);
/// ```
pub fn compile_library(libname: &str, files: &[&str], build_type: BuildType) {
    let target = env::var("TARGET").unwrap();

    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();
    let mut args:Vec<&str> = Vec::new();
    args.push(parse_triple(&target));

    if env::var_os("DEBUG").is_some() {
        args.push("-g");
    }

    let src = Path::new(&cargo_manifest_dir);

    let dst = Path::new(&out_dir);

    let mut objects = Vec::new();

    for file in files.iter() {
        let obj = dst.join(*file).with_extension("o");
        let mut cmd = Command::new("nasm");
        cmd.args(&args[..]);
        std::fs::create_dir_all(&obj.parent().unwrap()).unwrap();

        run(cmd.arg(src.join(*file)).arg("-o").arg(&obj));
        objects.push(obj);
    }

    match build_type {
        BuildType::STATIC => {
            let output_file = dst.join(format!("lib{}.a", libname));
            run(Command::new(ar()).arg("crus").arg(output_file).args(&objects[..]));
        },
        BuildType::SHARED => {
            let output_file = dst.join(format!("lib{}.so", libname));
            run(Command::new(ld()).args(&objects[..]).arg("-o").arg(output_file));   
        },
    };
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    let status = match cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit()).status() {
        Ok(status) => status,

        Err(e) => panic!("failed to spawn process: {}", e),
    };

    if !status.success() {
        panic!("nonzero exit status: {}", status);
    }
}

fn ar() -> String {
    env::var("AR").unwrap_or("ar".to_string())
}
fn ld() -> String {
    env::var("LD").unwrap_or("ld".to_string())
}
