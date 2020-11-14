//! Functions that open a text editor, vim.
use anyhow::{ensure, Context, Result};
use std::fs;
use std::str;
use subprocess::{Exec, ExitStatus};

/// Spawns an editor, waits for it to die, and returns the saved contents.
/// If the editor returns non-zero, this returns `None`. I'm assuming the
/// user doesn't actually want to change anything when that happens.
///
/// This only supports vim. Why would you need this to support anything else?
pub fn spawn(initial_contents: &str) -> Result<String> {
  let in_memory_filesystem = "/dev/shm";
  // Non-obvious, but important: `temporary_file` is deleted when it falls
  // out of scope.
  let temp =
    mktemp::Temp::new_file_in(in_memory_filesystem).with_context(|| {
      format!("creating a temporary file in {:?}", in_memory_filesystem)
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
  let edited_file_contents = fs::read(&temporary_file).with_context(|| {
    format!("reading the edited temporary file in {:?}", &temporary_file)
  })?;
  let edited_file_contents_as_str = str::from_utf8(&edited_file_contents)
    .with_context(|| "converting the edited file to utf-8")?;
  return Ok(String::from(edited_file_contents_as_str));
}
