use std::collections::BTreeMap;
use std::io::Write;

use camino::Utf8PathBuf;
use clap::Parser;
use testcontainers::ImageExt;
use testcontainers::core::IntoContainerPort;
use testcontainers::runners::AsyncRunner;

#[derive(clap::Parser)]
struct Args {
    #[clap(long, env = "DIESEL_CLI_TOOL")]
    diesel_cli_tool: Utf8PathBuf,

    #[clap(long, env = "DATABASE_IMAGE_LOAD_TOOL")]
    database_image_load_tool: Utf8PathBuf,

    #[clap(long, env = "OUTPUT_FILE")]
    output_file: Utf8PathBuf,

    schema_files: Vec<Utf8PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("running container load tool");

    let output = tokio::process::Command::new(args.database_image_load_tool)
        .output()
        .await
        .expect("failed to run database image load tool");

    if !output.status.success() {
        std::io::stderr().write_all(&output.stdout).expect("failed to write stdout");
        std::io::stderr().write_all(&output.stderr).expect("failed to write stderr");
        panic!("failed to run database image load tool");
    }

    let container_digest = String::from_utf8(output.stdout).expect("failed to read stdout");
    let container_digest = container_digest.lines().next().expect("failed to read container digest");
    let container_digest = container_digest.trim();
    let (image, tag) = container_digest.split_once(':').expect("failed to read container digest");

    log::info!("starting container: {image}:{tag}");

    let container = testcontainers::GenericImage::new(image, tag)
        .with_exposed_port(5432.tcp())
        .with_wait_for(testcontainers::core::WaitFor::message_on_either_std(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_PASSWORD", "scuffle")
        .start()
        .await
        .expect("failed to start container");

    let port = container.get_host_port_ipv4(5432).await.expect("failed to get host port");
    let url = container.get_host().await.expect("failed to get host");

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let db_url = format!("postgres://postgres:scuffle@{url}:{port}/scuffle");
    log::info!("database url: {db_url}");

    log::info!("applying migrations");
    let output = tokio::process::Command::new(&args.diesel_cli_tool)
        .env("DATABASE_URL", &db_url)
        .arg("database")
        .arg("reset")
        .output()
        .await
        .expect("failed to run diesel cli tool");

    if !output.status.success() {
        std::io::stderr().write_all(&output.stdout).expect("failed to write stdout");
        std::io::stderr().write_all(&output.stderr).expect("failed to write stderr");
        panic!("failed to run diesel cli tool");
    }

    let mut outputs = BTreeMap::new();

    for schema_file in args.schema_files {
        let content = std::fs::read_to_string(&schema_file).expect("failed to read schema file");
        outputs.insert(schema_file, content);
    }

    let json = serde_json::to_string_pretty(&outputs).expect("failed to write output file");

    std::fs::write(args.output_file, json).expect("failed to write output file");
}
