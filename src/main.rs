use tokio::process::Command;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut venv_setup = Command::new("virtualenv")
        .arg("venv")
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = venv_setup.wait().await?;

    println!("==> virtualenv status: {status}");

    let mut venv_requirements_setup = Command::new("venv/bin/pip")
        .arg("install")
        .arg("-r")
        .arg("requirements.txt")
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = venv_requirements_setup.wait().await?;

    println!("==> virtualenv requirements setup status: {status}");

    let kasa_device = config::parse_config()?;

    let mut kasa_data = Command::new("venv/bin/kasa")
        .arg("--host")
        .arg(kasa_device.ip.as_str())
        .arg("--username")
        .arg(kasa_device.username.as_str())
        .arg("--password")
        .arg(kasa_device.password.as_str())
        .spawn()
        .expect("Failed to setup virtualenv");

    let status = kasa_data.wait().await?;

    println!("==> kasa data status: {status}");

    Ok(())
}
