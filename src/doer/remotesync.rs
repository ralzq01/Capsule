extern crate serde_json;

use std::net::TcpStream;
use std::io::prelude::*;
use ssh2::Session;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::doer::base::MyDoer;
use crate::doer::base::Status;

pub struct RemoteSync<'a> {
  ip_port: &'a str,
  user: &'a str,
  password: &'a str,
  remote_dir: &'a str,
  local_dir: &'a str,
  tcp_conn: TcpStream,
  sess: Session,
}

impl<'a> RemoteSync<'a> {
  pub fn new(config: &'a HashMap<String, String>) -> RemoteSync {
    let ip_port = config.get("remote_ip_port").unwrap();
    let user = config.get("user").unwrap();
    let password = config.get("password").unwrap();
    let tcp_conn = TcpStream::connect(&ip_port).unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp_conn).unwrap();
    sess.userauth_password(user, password).expect(
      "Cannot connect to the remote server address."
    );
    assert!(sess.authenticated());
    RemoteSync {
      ip_port: ip_port,
      user: user,
      password: password,
      remote_dir: config.get("remote_dir").unwrap(),
      local_dir: config.get("local_dir").unwrap(),
      tcp_conn: tcp_conn,
      sess: sess,
    }
  }

  /// This will extract the last filename for a path
  /// e.g. input "aaa/bbb/cc.txt" or aaa/bbb/cc/
  ///      will get output of "cc.txt" or "cc"
  fn get_basename<'b> (filepath: &'b str) -> Result<&'b str, String> {
    let mut pieces = filepath.rsplit(|c| c == '/' || c == '\\');
    while let Some(p) = pieces.next() {
      if p.trim() == "" {
        continue
      } else {
        return Ok(p);
      }
    }
    Err("Can't parse the filepath".to_string())
  }

  /// This will generate correct filepath for basic operations
  /// e.g. self.remote_dir = "/path/to/remote/dir_name"
  /// e.g. self.local_dir = "/path/to/local/dir_name"
  /// e.g.     where "dir_name" should be same
  /// e.g. filepath: "/path/to/local/dir_name/dir1/file1.rs"
  /// e.g. output should be: "/path/to/remote/dir_name/dir1/file1.rs"
  fn to_remote_filepath(&self, filepath: &str) -> String {
    // if filepath is empty, will return empty
    if filepath == "" {
      return String::from("");
    }
    let remote_base = RemoteSync::get_basename(self.remote_dir).unwrap();
    let filepath = str::replace(filepath, "\\", "/");
    let mut pieces= filepath.rsplit(remote_base);
    match pieces.next() {
      Some(p) => {
        let joint = Path::new(self.remote_dir).join(&p[1..]);
        let joint = joint.to_str().unwrap();
        return str::replace(joint, "\\", "/").to_string();
      },
      _ => String::from("Can't solve the path")
    }
  }

  /// Basic operations: run a bash cmd in remote
  /// return a cmd output result in OkText
  pub fn run_cmd(&self, cmd: &str) -> Status {
    let mut channel = self.sess.channel_session().unwrap();
    channel.exec(cmd).expect(
      "running command fails"
    );
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    Status::OkText(s)
  }

  /// Basic operations: send file to the remote
  /// e.g. local_filepath = "aaa/bbb/cc" or "aa\bdd\cc"
  /// e.g. remote_path = "file/dir/dd" or "file\dir\dd"
  /// e.g. the result will put the file content "cc" into remote_path: "file/dir/dd"
  fn send_file(&self, local_file_path: &str, remote_file_path: &str) -> Status {
    let contents = fs::read_to_string(local_file_path)
        .expect(format!("can't read from {}", local_file_path).as_str());
    let parent_dir = Path::new(remote_file_path).parent().unwrap();
    self.run_cmd(format!("mkdir -p {}", parent_dir.to_str().unwrap()).as_str());
    let lens = contents.len() as u64;
    let mut remote_file = self.sess.scp_send(Path::new(remote_file_path),
                                        0o644, lens, None).expect("create file failed");
    remote_file.write(contents.as_bytes()).expect("send file fail");
    Status::OkNone
  }

  /// Basic operations: change filename in the remote dir
  /// e.g. old_filename = "path/1/to/old.txt"
  /// e.g. new_filename = "path/2/to/new.txt"
  /// e.g. the result will be: "path/1/to/old.txt" renamed to "path/2/to/new.txt"
  fn rename_file(&self, old_filepath: &str, new_filepath: &str) -> Status {
    let cmd = format!("mv {} {}", 
                      old_filepath,
                      new_filepath);
    let res = self.run_cmd(&cmd);
    match res {
      Status::OkText(_) => Status::OkNone,
      _ => res
    }
  }

  /// Basic operations: remove filename in the remote dir
  /// e.g. remote_path = "path/to/dir" or "path/to/file.txt"
  /// e.g. remove all the file (and dirs recursively) under remote_path
  fn remove_file(&self, file_path: &str, is_recursive: bool) -> Status {
    let mut cmd = format!("rm ");
    if is_recursive {
      cmd.push_str("-r ");
    }
    cmd.push_str(file_path);
    let res = self.run_cmd(&cmd);
    match res {
      Status::OkText(out) => {
        if out.as_str() == "" {
          return Status::OkNone;
        } else {
          return Status::Error(out);
        }
      },
      _ => {
        return res
      }
    }
  }
}

impl<'a> MyDoer for RemoteSync<'a> {
  /// recv an json string from upper watcher
  /// sync the files from the local to remote
  fn get(&self, input: String) -> Status {
    let event: Value = serde_json::from_str(&input).unwrap();
    if event["event"] == "FileWatcher" {
      let local_new_file = event["new"].as_str().unwrap();
      let remote_new_file = self.to_remote_filepath(local_new_file);
      let remote_old_file = self.to_remote_filepath(event["old"].as_str().unwrap());
      let event_type = event["type"].as_str().unwrap();
      if event_type == "Create" || event_type == "Write" {
        return self.send_file(local_new_file, &remote_new_file);    
      } else if event_type == "Rename" {
        return self.rename_file(&remote_old_file, &remote_new_file);
      } else if event_type == "Remove" {
        return self.remove_file(&remote_old_file, true);
      }
    } else {
      Status::Error("Currently Only support FileWatcher".to_string());
    }
    Status::OkNone
  }
}

