//! Functions that open a text editor, vim.
use anyhow::{ensure, Context, Result};
use os_type::{current_platform, OSType};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;
use subprocess::{Exec, ExitStatus};

/// We want to put the plaintext in a temporary file so the editor can edit it.
/// It shouldn't live on disk because those don't really get deleted when you rm
/// them. On linux this is easy to fix: just plop files into /dev/shm, a tmpfs.
/// But osx doesn't ship with a RAM-only filesystem enabled by default, so we make
/// our own.
///
/// 1) hdiutil attach -nomount ram://$((2 * 1024 * 10)) # 10 = 10 MB of space
///      - returns something like "/dev/disk2" on stdout
/// 2) diskutil eraseVolume HFS+ RamDisk /dev/disk2
/// 3) now you can put temp stuff in /Volumes/RamDisk
/// 4) When you're done, run hdiutil detach /dev/disk2
enum InMemoryFS {
  Linux { path: PathBuf },
  OSX { volume: PathBuf, device: PathBuf },
}

/// https://stackoverflow.com/questions/2033362/does-os-x-have-an-equivalent-to-dev-shm
fn create_osx_tmpfs() -> Result<InMemoryFS> {
  let mb_to_allocate = 10;
  let attach_result = Exec::cmd("hdiutil")
    .arg("attach")
    .arg("-nomount")
    // idk why 2*1024 = mb, but the internet says it does:
    .arg(format!("ram://{}", 2 * 1024 * mb_to_allocate))
    .capture()
    .with_context(|| "creating a temporary volume")?;
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
      // Windows has linux-emulation mode, so there are only two operating
      // systems worth supporting.
      OSType::OSX => create_osx_tmpfs(),
      _ => Ok(InMemoryFS::Linux {
        path: PathBuf::from("/dev/shm"),
      }),
    }
  }

  fn tmpdir(&self) -> &PathBuf {
    match &self {
      InMemoryFS::Linux { path } => &path,
      InMemoryFS::OSX { volume, device: _ } => volume,
    }
  }
}

impl Drop for InMemoryFS {
  fn drop(&mut self) {
    match &self {
      InMemoryFS::Linux { path: _ } => {}
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
  fs::read(&temporary_file).with_context(|| {
    format!("reading the edited temporary file in {:?}", &temporary_file)
  })
}
