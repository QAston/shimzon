use clap::{Arg, App, SubCommand};
use std::path::PathBuf;
use std::path::Path;

use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct ShimConf {
    path: String,
    args: Option<String>,
}

fn gen_shim(source_exe: &str, dest_dir: &str, shim_name: &str) -> i32 { 
    let shim_executable = include_bytes!(concat!(env!("OUT_DIR"), "/shim.exe"));
    // check if source exists
    match std::fs::metadata(Path::new(source_exe)) {
        Ok(meta) => {
            if !meta.is_file() {
                panic!("<source-exe> is not a file")
            }
        }
        Err(err) => {
            // allow some errors to allow linking to IO_REPARSE_TAG_APPEXECLINK "files"
            if err.kind() == std::io::ErrorKind::NotFound {
                panic!("<source-exe> not found")
            }
        }
    }
    let dest_dir = Path::new(&dest_dir);

    if Path::new(shim_name).extension().unwrap_or(std::ffi::OsStr::new("")) != "exe" {
        panic!("unsupported target file name")
    }
    
    std::fs::create_dir_all(dest_dir).unwrap();
    let shim_path = [&dest_dir, Path::new(shim_name)].iter().collect::<PathBuf>();
    let shim_path = Path::new(&shim_path);
    let shim_conf_path = shim_path.with_extension("shim");    
    std::fs::write(shim_path, shim_executable).unwrap();

    let mut source_path = std::env::current_dir().unwrap();
    source_path.push(source_exe);

    let source_path = if Path::new(source_exe).is_absolute() {
        Path::new(source_exe)
    } else {
        Path::new(&source_path)
        // todo: .canonicalize();?
    };
    
    let shim_conf = ShimConf {
        path: source_path.to_string_lossy().to_string(),
        args: None,
    };

    let toml = toml::to_string(&shim_conf).unwrap();
    std::fs::write(shim_conf_path, toml).unwrap();
    println!("created shim {}->{}", shim_path.to_str().unwrap(), source_path.to_str().unwrap());
    return 0
}

fn find_shim_files_recursively(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(find_shim_files_recursively(&path));
        } else if path.is_file() {
            if path.extension().unwrap_or(std::ffi::OsStr::new("")) == "shim" {
                files.push(path.to_string_lossy().to_string());
            }
        }
    }
    files
}

fn update_shim(dir: &str) -> i32 { 
    // check if source exists
    match std::fs::metadata(Path::new(dir)) {
        Ok(meta) => {
            if !meta.is_dir() {
                panic!("<source-exe> is not a file")
            }
        }
        Err(err) => {
            // allow some errors to allow linking to IO_REPARSE_TAG_APPEXECLINK "files"
            if err.kind() == std::io::ErrorKind::NotFound {
                panic!("<source-exe> not found")
            }
        }
    }
    let dest_dir = Path::new(dir);

    let shim_executable = include_bytes!(concat!(env!("OUT_DIR"), "/shim.exe"));

    let shim_files = find_shim_files_recursively(&dest_dir);

    for file in shim_files {
        //let shim_conf: ShimConf = toml::from_str(&std::fs::read_to_string(&file).unwrap()).unwrap();
        //let source_exe = Path::new(&shim_conf.path);
        let shim_path = Path::new(&file);
        let shim_path = shim_path.with_extension("exe");
        let shim_path = shim_path.as_path();
        let shim_path = [&dest_dir, Path::new(shim_path)].iter().collect::<PathBuf>();
        let shim_path = Path::new(&shim_path);
        std::fs::write(shim_path, shim_executable).unwrap();
        println!("updated shim {}", shim_path.to_str().unwrap());
    }
    return 0
}

fn main() {

    let matches = App::new(std::env::current_exe().unwrap().to_str().unwrap())
            .subcommand(SubCommand::with_name("add")
                        .about("create a shim")
                        .arg(Arg::with_name("source-exe")
                            .required(true)
                            .index(1)
                            .help("location of the shimmed exec"))
                        .arg(Arg::with_name("dest-dir")
                            .short("d")
                            .long("dest-dir")
                            .required(false)
                            .takes_value(true)
                            .help("where to write the shim"))
                        .arg(Arg::with_name("shim-name")
                            .short("n")
                            .long("shim-name")
                            .required(false)
                            .takes_value(true)
                            .help("how to name the resulting shim")))
            .subcommand(SubCommand::with_name("update")
                        .about("update the executable of existing .shim file")
                        .arg(Arg::with_name("dir")
                            .short("d")
                            .long("dir")
                            .required(true)
                            .takes_value(true)
                            .help("directory with .shims to update")))
            .get_matches();

    match matches.subcommand() {
        // shinzon add ./path.exe bucket
        ("add", Some(matches)) => {
            let source = matches.value_of("source-exe").unwrap();
            let dest = matches.value_of("dest-dir").unwrap_or("");
            let target = matches.value_of("shim-name").unwrap_or(Path::new(&source).file_name().unwrap().to_str().unwrap());
            std::process::exit(gen_shim(&source, &dest, &target));
        },
        ("update", Some(matches)) => {
            let dest = matches.value_of("dir").unwrap_or("");
            std::process::exit(update_shim(&dest));
        },
        _ => {
            println!("Missing arguments, try --help");
        }
    }
}
