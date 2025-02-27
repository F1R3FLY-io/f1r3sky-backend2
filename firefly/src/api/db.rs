use std::process::Command;

pub fn dump(db_url: String) -> anyhow::Result<String> {
    let mut command = Command::new("pg_dump");

    command.arg("--data-only");
    command.arg("--column-inserts");
    command.arg("--exclude-schema=public");
    command.arg(db_url);

    let output = command.output()?;
    if output.status.success() {
        String::from_utf8(output.stdout).map_err(Into::into)
    } else {
        Err(anyhow::anyhow!(
            "error from pg_dump: {:?}",
            String::from_utf8(output.stderr)
        ))
    }
}
