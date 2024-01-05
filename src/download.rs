pub fn download_and_uncompress(
    url: &str,
    output_path: impl AsRef<std::path::Path>,
    force_download: bool,
) -> anyhow::Result<()> {
    let output_path = output_path.as_ref();

    // Determine the output path for the compressed file based on the URL
    let output_path_compressed = output_path
        .parent()
        .unwrap_or(output_path)
        .join(url.split('/').last().unwrap_or_else(|| {
            output_path
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("out_compressed"))
                .to_str()
                .unwrap_or("out_compressed")
        }))
        .with_extension(url.split('.').last().unwrap_or("zip"));

    // Skip download if not necessary
    let should_download = match (
        force_download,
        output_path.try_exists()?,
        output_path_compressed.try_exists()?,
    ) {
        // If force_download is true, always download
        (true, ..) => true,
        // If the output_path exists and output_path_compressed does not, nothing needs to be done
        (.., true, false) => false,
        _ => true,
    };
    if !should_download {
        return Ok(());
    }

    // Download
    tokio::runtime::Runtime::new()?.block_on(download_binary_file(url, &output_path_compressed))?;

    // Decompress
    compress_tools::uncompress_archive(
        std::fs::File::open(&output_path_compressed)?,
        output_path,
        compress_tools::Ownership::Preserve,
    )?;

    // Remove the compressed file (no longer needed)
    std::fs::remove_file(&output_path_compressed)?;

    Ok(())
}

pub async fn download_binary_file(
    url: &str,
    output: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let response = reqwest::get(url).await?;
    let mut reader = std::io::Cursor::new(response.bytes().await?);

    let mut output_file = std::fs::File::create(output.as_ref())?;
    std::io::copy(&mut reader, &mut output_file)?;

    Ok(())
}
