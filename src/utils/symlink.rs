pub fn create_symlink(
    original: impl AsRef<std::path::Path>,
    link: impl AsRef<std::path::Path>,
    override_existing_symlink: bool,
) -> anyhow::Result<()> {
    let link = link.as_ref();

    // Make sure the link path does not already exist as a file or directory
    if link.try_exists()? {
        if link.is_dir() {
            anyhow::bail!(
                "Unable to create symlink `{}`: a directory already exists at this path.",
                link.display(),
            );
        } else if link.is_file() {
            anyhow::bail!(
                "Unable to create symlink `{}`: a file already exists at this path.",
                link.display(),
            );
        }
    }
    if link.is_symlink() {
        if override_existing_symlink {
            std::fs::remove_file(link)?;
        } else {
            anyhow::bail!(
                "Unable to create symlink `{}`: a symlink already exists at this path.",
                link.display(),
            );
        }
    }

    // Make sure the parent directory exists
    if let Some(parent) = link.parent() {
        if !parent.try_exists()? {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Create the symlink
    #[cfg(target_family = "unix")]
    std::os::unix::fs::symlink(original, link)?;
    #[cfg(target_family = "windows")]
    std::os::windows::fs::symlink_dir(original, link)?;

    Ok(())
}
