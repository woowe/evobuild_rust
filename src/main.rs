use std::env;
use std::path::{Path, PathBuf};
use std::fs::metadata;

extern crate pgs_files;
use pgs_files::passwd;

const BASEDIR: &'static str = "/var/cache/ evobuild";
const INSTDIR: &'static str = "/var/lib/evobuild/roots";
const DLDIR: &'static str = "/var/lib/evobuild/images";
const PROFILE: &'static str = "main-x86_64";

const ARCHIVEDIR: &'static str = "/var/lib/evobuild/archives";
const ARCHIVEDIR_TGT: &'static str = "/var/cache/eopkg/archives";


const CCACHE_DIR: &'static str = "/var/lib/evobuild/ccache";
const CCACHE_TGT: &'static str = "/root/.ccache";

const PACKAGE_DIR: &'static str = "/var/lib/evobuild/packages";
const PACKAGE_TGT: &'static str = "/var/cache/eopkg/packages";

const IMG_URI: &'static str = "https://www.solus-project.com/image_root/";
const known_profiles: [&'static str; 2] = ["main-x86_64", "unstable-x86_64"];
const IMG_SUFFIX: &'static str = ".img";
const IMG_DL_SUFFIX: &'static str = ".img.xz";

// # Lock file descriptor
// lockfd = None
// did_mount = False
//
// update_mode = False

// TODO cleanup all the panic!s with something better, more graceful
fn config_path() -> PathBuf {
    // ypkg config path
    let pwd_entry = match env::var_os("SUDO_UID") {
        Some(ref uid) => {
            let pwuid = match uid.to_str(){
                Some(id) => id,
                None => panic!("Could parse uid as string!")
            };
            passwd::get_entry_by_uid(pwuid.parse::<u32>().unwrap())
        },
        None => {
            println!("not running in sudo mode...");
            None
        }
    };

    let home_dir = match pwd_entry {
        Some(entry) => entry.dir,
        None => match env::home_dir() {
            Some(ref p) => p.to_str().unwrap().to_string(),
            None => panic!("Couldn't get home directory")
        }
    };

    println!("{:?}", home_dir);
    let conf_path = Path::new(&home_dir).join(".solus").join("packager");
    match metadata(&conf_path) {
        Ok(_) => conf_path,
        Err(_) => Path::new(&home_dir).join(".solus").join("packager")
    }
}

fn main() {
    println!("Hello, world!");

    println!("{:?}", config_path());
}
