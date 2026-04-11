use tokio::process::Command;

/// Operations for setting up execution environment.

pub(crate) async fn venv() -> Result<(), Box<dyn std::error::Error>> {
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

    Ok(())
}
