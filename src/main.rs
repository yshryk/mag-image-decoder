use mag_image_decoder::Decoder;
use std::fs::File;
use std::io::BufReader;
use simple_logger;
use log::info;
use structopt::StructOpt;
use std::path::PathBuf;
use std::{fs, process};

#[derive(StructOpt, Debug)]
#[structopt(name = "magdecode", author = "", about = "\
MAG image decoder")]
struct Opt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: u8,

    /// Specify the output directory
    #[structopt(short = "o", long = "outdir", name = "DIR", parse(from_os_str))]
    out_dir: Option<PathBuf>,

    /// Files to process
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
}

fn main() {
    match run(Opt::from_args()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn run(opt: Opt) -> Result<(), String> {
    if opt.verbose > 0 {
        simple_logger::init().expect("logger init error");
    }

    if let Some(dir) = &opt.out_dir {
        if !dir.exists() {
            info!("create directory: '{}'", dir.display());
            fs::create_dir_all(dir)
                .map_err(|e| format!("'{}': {}", dir.display(), e))?;
        }
    }

    if opt.files.is_empty() {
        Err("No input file specified.".to_owned())
    } else {
        for input_file in &opt.files {
            info!("input_file: {}", input_file.display());
            let reader = BufReader::new(File::open(input_file)
                .map_err(|e| format!("'{}': {}", input_file.display(), e))?);
            let decoder = Decoder::new(reader).map_err(|e| format!("{}", e))?;
            let header = decoder.info();
            info!("{:?}", header);

            let mut output_path = match &opt.out_dir {
                Some(dir) => dir.clone(),
                None => PathBuf::new()
            };
            output_path.push(input_file);
            output_path.set_extension("png");
            info!("output_path: '{}'", output_path.display());
            let img = decoder.decode().map_err(|e| format!("{}", e))?;
            img.save(output_path).map_err(|e| format!("failed to save: {}", e))?;
            info!("ok");
        }
        Ok(())
    }
}
