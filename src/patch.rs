pub fn create_file_patch(
    original_path: impl AsRef<std::path::Path>,
    modified_path: impl AsRef<std::path::Path>,
    patch_path: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let original_path = original_path.as_ref();
    let modified_path = modified_path.as_ref();
    let patch_path = patch_path.as_ref();

    // Read the original and modified files
    let original_content = std::fs::read_to_string(original_path)?;
    let modified_content = std::fs::read_to_string(modified_path)?;

    // Create the patch
    let patch = diffy::create_patch(&original_content, &modified_content);
    let mut patch_content = patch.to_string();

    // Update the original and modified paths in the patch if they have a common relative tail
    let common_path: std::path::PathBuf = original_path
        .canonicalize()?
        .components()
        .rev()
        .zip(modified_path.canonicalize()?.components().rev())
        .take_while(|(a, b)| a == b)
        .unzip::<_, _, Vec<_>, Vec<_>>()
        .0
        .into_iter()
        .rev()
        .collect();
    if !common_path.to_string_lossy().is_empty() && common_path.is_relative() {
        let common_path = std::path::PathBuf::from(".").join(common_path);
        patch_content = patch_content
            .replacen(
                "--- original",
                &format!("--- {}", common_path.to_string_lossy()),
                1,
            )
            .replacen(
                "+++ modified",
                &format!("+++ {}", common_path.to_string_lossy()),
                1,
            );
    }

    // Make sure the parent directory exists
    if let Some(parent) = patch_path.parent() {
        if !parent.try_exists()? {
            std::fs::create_dir_all(parent).unwrap();
        }
    }

    // Write the patch into a file
    std::fs::write(patch_path, &patch_content)?;

    Ok(())
}

pub fn create_file_patches(
    original_dir: impl AsRef<std::path::Path>,
    modified_dir: impl AsRef<std::path::Path>,
    patch_dir: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    let original_dir = original_dir.as_ref();
    let modified_dir = modified_dir.as_ref();
    let patch_dir = patch_dir.as_ref();
    // Iterate over all modified files
    walkdir::WalkDir::new(modified_dir)
        .into_iter()
        .filter(|entry| {
            entry
                .as_ref()
                .map(|entry| entry.file_type().is_file())
                .unwrap_or(false)
        })
        .map(|entry| entry.unwrap().path().to_path_buf())
        .try_for_each(|modified_file| {
            // Get path to the original file
            let original_file = original_dir.join(
                modified_file
                    .canonicalize()
                    .unwrap()
                    .strip_prefix(modified_dir.canonicalize().unwrap().as_os_str())
                    .unwrap(),
            );
            // Get path to the output patch file
            let patch_file = patch_dir.join(
                modified_file
                    .canonicalize()
                    .unwrap()
                    .strip_prefix(modified_dir.canonicalize().unwrap().as_os_str())
                    .unwrap()
                    .to_string_lossy()
                    .to_string()
                    + ".patch",
            );
            // Create the patch
            create_file_patch(original_file, &modified_file, patch_file)
        })
}

pub fn apply_file_patch(
    patch_path: impl AsRef<std::path::Path>,
    original_path: impl AsRef<std::path::Path>,
    target_path: impl AsRef<std::path::Path>,
    rerun_if_patch_changed: bool,
) -> Result<(), std::io::Error> {
    let patch_path = patch_path.as_ref();
    let target_path = target_path.as_ref();

    // Inform cargo to rerun this build script if the patch file changes
    if rerun_if_patch_changed {
        println!("cargo:rerun-if-changed={}", patch_path.display());
    }

    // Parse the patch
    let patch_string = std::fs::read_to_string(patch_path)?;
    let patch = diffy::Patch::from_str(&patch_string).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to parse patch: {err}"),
        )
    })?;

    // Read the original file
    let content = std::fs::read_to_string(original_path)?;

    // Apply the patch
    let patched_content = diffy::apply(&content, &patch).map_err(|err| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("failed to apply patch: {err}"),
        )
    })?;

    // If the target file already exists and the patched content is the same as the target content, skip
    if target_path.is_file() {
        let target_content = std::fs::read_to_string(target_path)?;
        if patched_content == target_content {
            return Ok(());
        }
    }

    // Make sure the parent directory exists
    if let Some(parent) = target_path.parent() {
        if !parent.try_exists()? {
            std::fs::create_dir_all(parent).unwrap();
        }
    }

    // Write the patched content to the target path
    std::fs::write(target_path, patched_content)?;

    Ok(())
}

pub fn apply_file_patch_in_place(
    patch_path: impl AsRef<std::path::Path>,
    target_path: impl AsRef<std::path::Path>,
    rerun_if_patch_changed: bool,
) -> Result<(), std::io::Error> {
    apply_file_patch(
        patch_path,
        &target_path,
        &target_path,
        rerun_if_patch_changed,
    )
}

pub fn apply_file_patches(
    patch_dir: impl AsRef<std::path::Path>,
    original_dir: impl AsRef<std::path::Path>,
    target_dir: impl AsRef<std::path::Path>,
    rerun_if_patch_changed: bool,
) -> Result<(), std::io::Error> {
    let patch_dir = patch_dir.as_ref();
    let original_dir = original_dir.as_ref();
    let target_dir = target_dir.as_ref();

    // Iterate over all patch files
    walkdir::WalkDir::new(patch_dir)
        .into_iter()
        .filter(|entry| {
            entry
                .as_ref()
                .map(|entry| {
                    entry.file_type().is_file()
                        && entry
                            .path()
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .ends_with("patch")
                })
                .unwrap_or(false)
        })
        .map(|entry| entry.unwrap().path().to_path_buf())
        .try_for_each(|patch| {
            // Get relative path of the patched file
            let patched_file_relative = patch
                .canonicalize()
                .unwrap()
                .strip_prefix(patch_dir.canonicalize().unwrap().as_os_str())
                .unwrap()
                .with_file_name(
                    patch
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .strip_suffix(".patch")
                        .unwrap(),
                );

            // Get the paths to the original and target files
            let original_file = original_dir.join(&patched_file_relative);
            let target_file = target_dir.join(&patched_file_relative);

            // Apply the patch
            apply_file_patch(&patch, original_file, target_file, rerun_if_patch_changed)
        })
}

pub fn apply_file_patches_in_place(
    patch_dir: impl AsRef<std::path::Path>,
    target_dir: impl AsRef<std::path::Path>,
    copy_original: bool,
    rerun_if_patch_changed: bool,
) -> Result<(), std::io::Error> {
    let patch_dir = patch_dir.as_ref();
    let target_dir = target_dir.as_ref();
    // Iterate over all patch files
    walkdir::WalkDir::new(patch_dir)
        .into_iter()
        .filter(|entry| {
            entry
                .as_ref()
                .map(|entry| {
                    entry.file_type().is_file()
                        && entry
                            .path()
                            .extension()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default()
                            .ends_with("patch")
                })
                .unwrap_or(false)
        })
        .map(|entry| entry.unwrap().path().to_path_buf())
        .try_for_each(|patch| {
            // Get relative path of the patched file
            let patched_file_relative = patch
                .canonicalize()
                .unwrap()
                .strip_prefix(patch_dir.canonicalize().unwrap().as_os_str())
                .unwrap()
                .with_file_name(
                    patch
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .strip_suffix(".patch")
                        .unwrap(),
                );

            // Get the path to the target file
            let target_file = target_dir.join(patched_file_relative);

            // Determine the path to the original file (either the target file or a copy of it)
            let original_file = if copy_original {
                // If requested, create a copy of the target file and treat it as the original file
                let original_file = target_file.with_extension(
                    target_file
                        .extension()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default()
                        .to_owned()
                        + ".original",
                );
                if !original_file.is_file() {
                    std::fs::copy(&target_file, &original_file).unwrap();
                }
                original_file
            } else {
                // Otherwise, use the target file as the original file
                target_file.clone()
            };

            // Apply the patch
            apply_file_patch(&patch, original_file, &target_file, rerun_if_patch_changed)
        })
}

pub fn is_file_patch_applied(
    patch_path: impl AsRef<std::path::Path>,
    original_path: impl AsRef<std::path::Path>,
    target_path: impl AsRef<std::path::Path>,
) -> bool {
    // Parse the patch
    let patch_string = std::fs::read_to_string(patch_path).unwrap();
    let patch = diffy::Patch::from_str(&patch_string).unwrap();

    // Read the original file
    let content = std::fs::read_to_string(original_path).unwrap();

    // Apply the patch
    let patched_content = diffy::apply(&content, &patch).unwrap();

    // Read the target file
    let target_content = std::fs::read_to_string(target_path).unwrap();

    // Compare the patched content with the target content
    patched_content == target_content
}
