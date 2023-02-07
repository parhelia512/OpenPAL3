pub mod cpk;
pub mod fmb;
pub mod imd;
pub mod memory_file;
pub mod pkg;
pub mod plain_fs;
pub mod sfb;

use std::{
    fs,
    io::{Read, Seek},
    path::{Path, PathBuf},
};

use mini_fs::{LocalFs, MiniFs};

use crate::fs::{
    cpk::CpkFs, fmb::fmb_fs::FmbFs, imd::imd_fs::ImdFs, pkg::pkg_fs::PkgFs, sfb::sfb_fs::SfbFs,
};

pub fn init_virtual_fs<P: AsRef<Path>>(local_asset_path: P, pkg_key: Option<&str>) -> MiniFs {
    let local = LocalFs::new(local_asset_path.as_ref());
    let vfs = MiniFs::new(false).mount("/", local);
    let vfs = mount_packages_recursive(
        vfs,
        local_asset_path.as_ref(),
        &PathBuf::from("./"),
        pkg_key,
    );

    vfs
}

fn mount_packages_recursive(
    mut vfs: MiniFs,
    local_path: &Path,
    relative_path: &Path,
    pkg_key: Option<&str>,
) -> MiniFs {
    let path = local_path.join(relative_path);
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            let entry = entry.unwrap();
            let new_path = relative_path.join(entry.file_name());
            vfs = mount_packages_recursive(vfs, local_path, &new_path, pkg_key.clone());
        }
    } else {
        let vfs_path = PathBuf::from("/").join(relative_path.with_extension(""));
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("cpk") => {
                log::debug!("Mounting {:?} <- {:?}", &vfs_path, &path);
                vfs = vfs.mount(vfs_path, CpkFs::new(path).unwrap())
            }
            Some("fmb") => {
                log::debug!("Mounting {:?} <- {:?}", &vfs_path, &path);
                vfs = vfs.mount(vfs_path, FmbFs::create(path).unwrap())
            }
            Some("imd") => {
                log::debug!("Mounting {:?} <- {:?}", &vfs_path, &path);
                vfs = vfs.mount(vfs_path, ImdFs::create(path).unwrap())
            }
            Some("sfb") => {
                log::debug!("Mounting {:?} <- {:?}", &vfs_path, &path);
                vfs = vfs.mount(vfs_path, SfbFs::create(path).unwrap())
            }
            Some("pkg") => match pkg_key {
                None => log::debug!("Didn't mount {:?} as pkg key is not provided", &path),
                Some(key) => {
                    log::debug!("Mounting {:?} <- {:?}", &vfs_path, &path);
                    vfs = vfs.mount(vfs_path, PkgFs::new(path, key).unwrap())
                }
            },
            _ => {}
        }
    }

    vfs
}

pub trait SeekRead: Read + Seek {}
impl<T> SeekRead for T where T: Read + Seek {}