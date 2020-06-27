use std::path::Path;

extern crate clap;
use clap::{Arg, App};

mod pbzx;

fn main() -> std::io::Result<()> {
    let matches = App::new("pbzx-apple")
        .version("1.0.0")
        .author("Kenan Sulayman <kenan@sig.dev>")
        .arg(Arg::with_name("DIR")
            .short("o")
            .long("outdir")
            .value_name("DIR")
            .help("Output directory")
            .takes_value(true)
            .default_value("out"))
        .arg(Arg::with_name("FILE")
            .help("Path of the ftab file to process")
            .required(true)
            .index(1))
        .arg(Arg::with_name("verbose")
            .short("v")
            .multiple(true)
            .help("Lists tags of dumped entries and the size of their data"))
        .arg(Arg::with_name("force")
            .short("f")
            .help("Ignore existing directory and write into it."))
        .get_matches();

    let file_path = matches.value_of("FILE").unwrap();
    let outdir = Path::new(matches.value_of("DIR").unwrap());

    let force = matches.is_present("force");
    let _verbose = matches.is_present("verbose");

    let created_dir = std::fs::create_dir(outdir);

    if !force && !created_dir.is_ok() {
        println!(
            "Error: directory {:?} exists. Pass '-f' if you want to proceed anyway.",
            outdir,
        );

        return Ok(());
    }

    let mut file_buf = match std::fs::File::open(file_path) {
        Ok(buf) => buf,
        Err(err) => {
            println!("Error: {}.", err);

            return Ok(());
        }
    };

    let reported_file_len = file_buf.metadata()?.len();

    let pbzx_file = pbzx::proces(
        &mut file_buf,
        reported_file_len,
    )?;

    pbzx_file
        .entries
        .iter()
        .enumerate()
        .for_each(|(i, entry)|
            { std::fs::write(format!("{}", i), &entry.data); }
        );

    /*let ftab = pbzx::proces(&mut file_buf)?;

    let mut total_bytes = 0;
    let num_files = ftab.entries.len();

    for entry in ftab.entries {
        if verbose {
            println!("{}: {} bytes", entry.tag, entry.data.len());
        }

        total_bytes += entry.data.len();

        std::fs::write(
            outdir.join(entry.tag),
            &entry.data,
        )?;
    }

    println!(
        "✔ wrote {} files with total of {} bytes",
        num_files,
        total_bytes,
    );*/

    return Ok(());
}
