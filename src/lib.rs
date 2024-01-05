//! The `build_different` crate that can reduce the boilerplace in build scripts for some of your crates.
//! It is especially useful for `*sys` crates that need to download C/C++ source code or binaries.

pub mod download;
pub mod patch;
pub mod utils;

pub use download::download_and_uncompress;
pub use patch::{
    apply_file_patch, apply_file_patch_in_place, apply_file_patches, apply_file_patches_in_place,
    create_file_patch, create_file_patches, is_file_patch_applied,
};
pub use utils::{cache::cache_dir, parse::parse_bool_env, symlink::create_symlink};
