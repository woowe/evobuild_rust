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

fn work_dir() -> PathBuf {
    Path::new(BASEDIR).join("work")
}

fn upper_dir() -> PathBuf {
    //  Return upper_dir for overlayfs
    Path::new(BASEDIR).join("transient")
}

fn lower_dir() -> PathBuf {
    // Return profiles lowerdir for overlayfs
    if update_mode {
        Path::new(INSTDIR).join(PROFILE)
    }
    else {
        Path::new(BASEDIR).join(PROFILE)
    }
}

fm union_dir() -> PathBuf {
    // Return the union (overlayfs) mount point
    if update_mode {
        lower_dir()
    }
    else {
        Path::new(BASEDIR).join("union")
    }
}

fn archive_dir() -> PathBuf {
    // Return the archive mount point
    Path::new(union_dir()).join(ARCHIVEDIR_TGT[1..])
}


fn ccache_dir() -> PathBuf {
    // Return the ccache mount point
    Path::new(union_dir()).join(CCACHE_TGT[1..])
}


fn package_dir() -> PathBuf {
    // Return the package mount point
    Path::new(union_dir()).join(PACKAGE_TGT[1..])
}


def image_path(dl=False):
    ''' Return path for the profile image '''
    return os.path.realpath(os.path.join(DLDIR, "%s%s" % (PROFILE, IMG_SUFFIX if not dl else IMG_DL_SUFFIX)))


def lock_file():
    ''' Current lock file '''
    if update_mode:
        return os.path.realpath(os.path.join(INSTDIR, "%s.lock" % PROFILE))
    else:
        return os.path.realpath(os.path.join(BASEDIR, "lock"))


def lock_root():
    ''' Lock the root and prevent concurrent manipulation '''
    global lockfd

    if os.path.exists(lock_file()):
        print "Lock file exists: %s" % lock_file()
        return False

    try:
        lockfd = open(lock_file(), "w")
        fcntl.lockf(lockfd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        lockfd.write("evobuild")
    except:
        return False
    return True


def unlock_root():
    ''' Unlock the root.. '''
    global lockfd

    if lockfd is not None:
        lockfd.close()
        try:
            if os.path.exists(lock_file()):
                os.unlink(lock_file())
        except Exception, e:
            print "Unable to clean lock file: %s" % lock_file()
            print e
            return False
        lockfd = None
    return True


def download_image():
    ''' Download the current profile image '''
    if os.path.exists(image_path(True)):
        return True
    if not os.path.exists(DLDIR):
        try:
            os.makedirs(DLDIR)
        except Exception, e:
            print "Unable to create directories: %s" % e
            return False

    uri = "%s%s%s" % (IMG_URI, PROFILE, IMG_DL_SUFFIX)
    cmd = "wget \"%s\" -O \"%s\"" % (uri, image_path(True))
    try:
        ret = os.system(cmd)
        if ret != 0:
            print "Invalid return code from wget. Aborting"
            if os.path.exists(image_path(True)):
                os.unlink(image_path(True))
            return False
    except Exception, e:
        print "Encountered error downloading image: %s" % e
        if os.path.exists(image_path(True)):
            os.unlink(image_path(True))
        return False
    return extract_image()


def do_mount(source,target,type=None,opts=None,bind=False):
    ''' Mount source over target, using optional type and options '''
    try:
        if type:
            type = "-t %s" % type
        else:
            type = ""
        if opts:
            opts = "-o %s" % opts
        else:
            opts = ""
        bindFlag = "--bind" if bind else ""
        if not os.path.exists(target):
            os.makedirs(target)
        # mount -o loop -t ext4 base.i
        ret = os.system("mount %s %s %s \"%s\" \"%s\"" % (opts,type,bindFlag, source,target))
        if ret != 0:
            print "Mount for %s->%s failed" % (source, target)
            return False
    except Exception, e:
        print "Execution of mount failed: %s" % e
        return False
    return True


def do_umount(point,force=False):
    ''' umount a given mountpoint, optionally with the --force flag '''
    try:
        cmd = "umount" if not force else "umount -f"
        ret = os.system("%s \"%s\"" % (cmd, point))
        if ret != 0:
            print "umount of %s failed" % point
            return False
    except Exception, e:
        print "Execution of umount failed: %s" % e
        return False
    return True


def extract_image():
    ''' Extract the profile image... '''
    if os.path.exists(image_path()):
        return True
    print "Extracting image"
    cwd = os.getcwd()
    try:
        os.chdir(DLDIR)
        ret = os.system("unxz \"%s%s\"" % (PROFILE, IMG_DL_SUFFIX))
        if ret != 0:
            print "Invalid return code from unxz. Aborting"
            return False
    except Exception, e:
        print "Encountered error extracting image: %s" % e
        return False
    os.chdir(cwd)
    return True


def run_chroot(cmd):
    ''' Run a command in the chroot '''
    try:
        ret = os.system("chroot \"%s\" %s" % (union_dir(), cmd))
        if ret != 0:
            print "Return code was %s" % ret
            return False
    except Exception, e:
        print "Chroot exception: %s" % e
        return False
    return True

def get_pkgname(fname):
    if fname.endswith("pspec.xml"):
        try:
            tree = ET.parse(fname)
            root = tree.getroot()

            name = root.findall("Source/Name")[0].text
            return name
        except Exception, e:
            print "Invalid XML error: %s" % e
            clean_exit(1)
    else:
        y= None
        try:
            f = open(fname, "r")
            y = yaml.load(f)
        except Exception, e:
            print "Unable to load %s: %s" % (fname, e)
            clean_exit(1)
        if y is None:
            print "Error processing %s" % fname
            clean_exit(1)
        if "name" not in y:
            print "%s does not provide mandatory name token" % fname
            clean_exit(1)
        return y["name"]


def clean_exit(code):
    ''' Try and ensure we cleanup... '''
    if did_mount:
        down_root()
    unlock_root()
    sys.exit(code)


fn main() {
    println!("Hello, world!");

    println!("{:?}", config_path());
}
