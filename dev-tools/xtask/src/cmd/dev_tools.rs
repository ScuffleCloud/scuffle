#[derive(Debug, Clone, clap::Parser)]
pub struct DevTools {}

fn debug_command(command: &mut std::process::Command) {
    println!(
        "Executing: {} {}",
        command.get_program().to_string_lossy(),
        command.get_args().map(|a| a.to_string_lossy()).collect::<Vec<_>>().join(" ")
    );
    command.stderr(std::process::Stdio::inherit());
    command.stdout(std::process::Stdio::inherit());
    command.output().unwrap();
}

impl DevTools {
    pub fn run(self) -> anyhow::Result<()> {
        println!("Installing rustup");

        const RUST_TOOLCHAINS: &[&str] = &["stable", "nightly"];

        for toolchain in RUST_TOOLCHAINS {
            debug_command(std::process::Command::new("rustup").arg("install").arg(toolchain));
        }

        println!("Installing rustup components");

        const COMPONENTS: &[&str] = &["clippy", "rustfmt", "llvm-tools-preview", "rust-src", "rust-docs"];

        for component in COMPONENTS {
            debug_command(
                std::process::Command::new("rustup")
                    .arg("component")
                    .arg("add")
                    .arg(component),
            );
        }

        println!("Installing cargo-binstall");

        debug_command(std::process::Command::new("cargo").arg("install").arg("cargo-binstall"));

        println!("Installing cargo-binstall packages");

        const BINSTALL_PACKAGES: &[&str] = &[
            "just",
            "cargo-llvm-cov",
            "cargo-nextest",
            "cargo-insta",
            "cargo-hakari",
            "miniserve",
            "taplo",
        ];

        for package in BINSTALL_PACKAGES {
            debug_command(std::process::Command::new("cargo").arg("binstall").arg(package));
        }

        // special case for cargo-sync-rdme
        debug_command(
            std::process::Command::new("cargo")
                .arg("binstall")
                .arg("cargo-sync-rdme")
                .arg("--git=https://github.com/TroyKomodo/cargo-sync-rdme.git")
                .arg("--force=-y"),
        );

        println!("Installation complete");

        Ok(())
    }
}
