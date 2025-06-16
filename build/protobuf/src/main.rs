use anyhow::Context;

fn usage() -> ! {
    let program = std::env::args().next().map(std::path::PathBuf::from);
    let program = program.as_ref().and_then(|p| p.file_name()).and_then(|o| o.to_str());
    let program = program.unwrap_or("file_concat");
    eprintln!("Usage: {program} <output> <inputs...>");
    std::process::exit(1)
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args_os();
    args.next();

    let Some(output) = args.next() else {
        usage();
    };

    let inputs: Vec<_> = args.collect();
    if inputs.is_empty() {
        usage();
    }

    let output_file = std::fs::File::options()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&output)
        .with_context(|| format!("output open: {}", output.display()))?;

    let mut output_writer = std::io::BufWriter::new(output_file);

    for input in inputs {
        let mut input_file = std::fs::File::open(&input).with_context(|| format!("input open: {}", input.display()))?;
        std::io::copy(&mut input_file, &mut output_writer).with_context(|| format!("input copy: {}", input.display()))?;
    }

    Ok(())
}
