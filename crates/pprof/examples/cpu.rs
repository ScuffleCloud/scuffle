#[cfg(unix)]
fn work() {
    use std::io::Write;

    use rand::Rng;

    let mut rng = rand::rng();

    let mut buf = vec![0u8; 1024 * 1024];
    rng.fill(buf.as_mut_slice());

    loop {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(buf.as_slice()).unwrap();
        buf = encoder.finish().unwrap();
    }
}

#[cfg(unix)]
fn main() {
    let cpu = scuffle_pprof::Cpu::new::<String>(1000, &[]);

    std::thread::spawn(work);

    let capture = cpu.capture(std::time::Duration::from_secs(10)).unwrap();

    std::fs::write("capture.pprof", capture).unwrap();
}

#[cfg(windows)]
fn main() {
    panic!("This example is not supported on Windows");
}
