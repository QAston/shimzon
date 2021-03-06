use serde_derive::Deserialize;
use clap::{Arg, App, SubCommand};
use std::path::PathBuf;
use std::path::Path;

use serde_derive::Serialize;

#[derive(Serialize)]
struct ShimConf {
    path: String,
    args: Option<String>,
}

#[derive(Deserialize)]
struct Config {
    base_out_path: String,
    shim_exe_path: String
}

fn gen_shim(source_exe: &str, dest_dir: &str, shim_name: &str, config: &Config, ) -> i32 { 
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
    let dest_dir = [&config.base_out_path, dest_dir].iter().collect::<PathBuf>();
    let dest_dir = Path::new(&dest_dir);

    if Path::new(shim_name).extension().unwrap_or(std::ffi::OsStr::new("")) != "exe" {
        panic!("unsupported target file name")
    }
    
    std::fs::create_dir_all(dest_dir).unwrap();
    let shim_path = [&dest_dir, Path::new(shim_name)].iter().collect::<PathBuf>();
    let shim_path = Path::new(&shim_path);
    let shim_conf_path = shim_path.with_extension("shim");
    

    let shim_binary = Path::new(&config.shim_exe_path);
    std::fs::copy(shim_binary, shim_path).unwrap();

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

fn main() {
    let exe_path = std::env::current_exe();
    let conf_path = exe_path.unwrap().parent().unwrap().join("config.toml");
    let toml_str = std::fs::read_to_string(conf_path).unwrap();
    let decoded: Config = toml::from_str(&toml_str).unwrap();

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
            .get_matches();

    if let Some(matches) = matches.subcommand_matches("add") {
        let source = matches.value_of("source-exe").unwrap();
        let dest = matches.value_of("dest-dir").unwrap_or("");
        let target = matches.value_of("shim-name").unwrap_or(Path::new(&source).file_name().unwrap().to_str().unwrap());
        std::process::exit(gen_shim(&source, &dest, &target, &decoded));
    } else {
        println!("Missing arguments, try --help");
    }

    // shinzon add ./path.exe bucket
}