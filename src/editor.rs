//! Functions that open a text editor, vim.
use anyhow::{bail, ensure, Context, Result};
use os_type::{current_platform, OSType};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use subprocess::{Exec, ExitStatus};

// argh. linux is easy -- just plop files into /dev/shm. but osx doesn't ship
// with a ramfs by default so we have to make one.
//
// 1) hdiutil attach -nomount ram://$((2 * 1024 * 10)) # 10 = 10 MB of space
//      - returns "/dev/disk2" on stdout
// 2) diskutil eraseVolume HFS+ RamDisk /dev/disk2
// 3) now you can put temp shit in /Volumes/RamDisk
//
// When you're done, run hdiutil detach /dev/disk2

enum InMemoryFS {
    Linux,
    OSX { volume: PathBuf, device: PathBuf },
}

/// https://stackoverflow.com/questions/2033362/does-os-x-have-an-equivalent-to-dev-shm
fn create_osx_tmpfs() -> Result<InMemoryFS> {
    let mb_to_allocate = 10;
    // idk why 2*1024 = mb, but the internet says it does:
    let attach_result = Exec::cmd("hdiutil")
        .arg("attach")
        .arg("-nomount")
        .arg(format!("ram://{}", 2 * 1024 * mb_to_allocate))
        .capture()
        .with_context(|| format!("creating a temporary volume"))?;
    let attach_stdout = String::from_utf8(attach_result.stdout)?;
    let device =
        PathBuf::from(&OsStr::from_bytes(attach_stdout.trim().as_bytes()));
    let volume_basepath = "lifecrypt";
    let volume = format!("/Volumes/{}", volume_basepath);
    Exec::cmd("diskutil")
        .arg("eraseVolume")
        .arg("HFS+")
        .arg(&volume_basepath)
        .arg(&device)
        .join()
        .with_context(|| {
            format!("erasing the temporary volume {}", &volume_basepath)
        })?;
    Ok(InMemoryFS::OSX {
        volume: PathBuf::from(volume),
        device,
    })
}

impl InMemoryFS {
    fn new() -> Result<InMemoryFS> {
        match current_platform().os_type {
            OSType::OSX => create_osx_tmpfs(),
            OSType::Redhat
            | OSType::Ubuntu
            | OSType::Debian
            | OSType::Arch
            | OSType::Manjaro
            | OSType::CentOS
            | OSType::OpenSUSE => Ok(InMemoryFS::Linux),
            OSType::Unknown => bail!("operating system not supported"),
        }
    }

    fn tmpdir(&self) -> PathBuf {
        match &self {
            &InMemoryFS::Linux => PathBuf::from("/dev/shm"),
            &InMemoryFS::OSX { volume, device: _ } => volume.clone(),
        }
    }
}

impl Drop for InMemoryFS {
    fn drop(&mut self) {
        match &self {
            InMemoryFS::Linux => {}
            InMemoryFS::OSX { volume: _, device } => {
                Exec::cmd("hdiutil")
                    .arg("detach")
                    .arg(&device)
                    .join()
                    .unwrap();
            }
        }
    }
}

/// Spawns an editor, waits for it to die, and returns the saved contents.
/// If the editor returns non-zero, this returns `None`. I'm assuming the
/// user doesn't actually want to change anything when that happens.
///
/// This only supports vim. Why would you need this to support anything else?
pub fn spawn(initial_contents: &[u8]) -> Result<Vec<u8>> {
    let in_memory_filesystem = InMemoryFS::new()?;
    let in_memory_fs_dir = in_memory_filesystem.tmpdir();
    // Non-obvious, but important: `temporary_file` is deleted when it falls
    // out of scope.
    let temp =
        mktemp::Temp::new_file_in(&in_memory_fs_dir).with_context(|| {
            format!("creating a temporary file in {:?}", &in_memory_fs_dir)
        })?;
    let temporary_file = temp.to_path_buf();
    fs::write(&temporary_file, initial_contents).with_context(|| {
        format!(
            "writing into newly-created temporary file {:?}",
            &temporary_file
        )
    })?;
    // "-n" disables swapfiles, which feels important for this application.
    let exit_status = Exec::cmd("vim")
        .arg("-n")
        .arg(&temporary_file)
        .join()
        .with_context(|| "editor execution failed")?;
    ensure!(
        exit_status == ExitStatus::Exited(0),
        "editor exited non-zero"
    );
    let edited_file_contents =
        fs::read(&temporary_file).with_context(|| {
            format!(
                "reading the edited temporary file in {:?}",
                &temporary_file
            )
        })?;
    return Ok(edited_file_contents);
}
