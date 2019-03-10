extern crate serde_json;

use std::net::TcpStream;
use std::io::prelude::*;
use ssh2::Session;
use serde_json::{json, Value};
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
  //sess: Session,
}

impl<'a> RemoteSync<'a> {
  pub fn new(config: &'a HashMap<String, String>) -> RemoteSync {
    let ip_port = config.get("remote_ip_port").unwrap();
    let user = config.get("user").unwrap();
    let password = config.get("password").unwrap();
    let tcp_conn = TcpStream::connect(&ip_port).unwrap();
    //let mut sess = Session::new().unwrap();
    //// connect to the remote
    //sess.handshake(&tcp_conn).unwrap();
    //sess.userauth_password(user, password).expect(
    //  "Cannot connect to the remote server address."
    //);
    //assert!(sess.authenticated());
    RemoteSync {
      ip_port: ip_port,
      user: user,
      password: password,
      remote_dir: config.get("remote_dir").unwrap(),
      local_dir: config.get("local_dir").unwrap(),
      tcp_conn: tcp_conn,
      //sess: sess,
    }
  }

  fn connect(&self) -> Session {
    //let tcp_conn = TcpStream::connect(self.ip_port).unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&self.tcp_conn).unwrap();
    sess.userauth_password(self.user, self.password).expect(
      "Cannot connect to the remote server address."
    );
    assert!(sess.authenticated());
    sess
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
    let sess = self.connect();
    let mut channel = sess.channel_session().unwrap();
    channel.exec(cmd).expect(
      "running command fails"
    );
    let mut s = String::new();
    channel.read_to_string(&mut s).unwrap();
    Status::OkText(s)
  }

  /// Basic operations: send file to the remote
  /// e.g. local_filepath = "aaa/bbb/cc" or "aa\bdd\cc"
  /// e.g. remote_path = "file/dir" or "file\dir\"
  /// e.g. the result will put the file "cc" into remote_path: "file/dir/cc"
  fn send_file(&self, local_file_path: &str, remote_path: &str) -> Status {
    let sess = self.connect();
    let contents = fs::read_to_string(local_file_path)
        .expect(format!("can't read from {}", local_file_path).as_str());
    let basename = RemoteSync::get_basename(local_file_path).unwrap();
    let remote_file_path = Path::new(remote_path).join(Path::new(basename));
    let lens = contents.len() as u64;
    let mut remote_file = sess.scp_send(&remote_file_path,
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
  fn remov_file(&self, file_path: &str, is_recursive: bool) -> Status {
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
      if event["type"] == "Create" {
        //self.send_file()
      }
    } else {
      println!("Currently RemoteSync only supports FileWatcher");
      Status::Error("Currently Only support FileWatcher".to_string());
    }

    Status::OkNone
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn construct_hashmap() -> HashMap<String, String> {
    let mut hm = HashMap::new();
    hm.insert(
      String::from("remote_ip_port"),
      String::from("xxx.xxx.xxx.xx:22")
    );
    hm.insert(
      String::from("user"),
      String::from("xxxx")
    );
    hm.insert(
      String::from("password"),
      String::from("xxxx")
    );
    hm.insert(
      String::from("remote_dir"),
      String::from("/home/zhiqilin/test1")
    );
    hm.insert(
      String::from("local_dir"),
      String::from("C::\\Users\\zhiqilin\\test1\\")
    );
    hm
  }

  #[test]
  fn test_password_connect() {
    let hm = construct_hashmap();
    let syncer = RemoteSync::new(&hm);
    syncer.connect();
  }

  #[test]
  fn test_upload_files() {
    let hm = construct_hashmap();
    let syncer = RemoteSync::new(&hm);
    let res = syncer.send_file("./README.md", "/home/v-zhilin/");
    match res {
      Status::OkNone => return,
      _ => panic!()
    }
  }

  #[test]
  fn test_run_cmd() {
    let hm = construct_hashmap();
    let syncer = RemoteSync::new(&hm);
    let res = syncer.run_cmd("echo aaabc");
    match res {
      Status::OkText(text) => {
        assert_eq!(text.as_str(), "aaabc\n");
      },
      _ => panic!("Can't recv returned message")
    }
  }

  #[test]
  fn test_get_basename() {
    let res = RemoteSync::get_basename("/path/test/test1.txt").unwrap();
    assert_eq!(res, "test1.txt");
    let res = RemoteSync::get_basename("c:\\Users\\abs\\test2.txt").unwrap();
    assert_eq!(res, "test2.txt");
    let res = RemoteSync::get_basename("/path/test_dir/").unwrap();
    assert_eq!(res, "test_dir");
    let res = RemoteSync::get_basename("/path/test_dir2/  ").unwrap();
    assert_eq!(res, "test_dir2");
  }

  #[test]
  fn test_to_remote_filepath() {
    let hm = construct_hashmap();
    let syncer = RemoteSync::new(&hm);
    let res = syncer.to_remote_filepath("C:\\Users\\zhiqilin\\test1\\test\\abc.txt");
    assert_eq!(res, "/home/zhiqilin/test1/test/abc.txt")
  }

  #[test]
  fn test_rename_file() {
    let hm = construct_hashmap();
    let syncer = RemoteSync::new(&hm);
    let res = syncer.rename_file("~/README.md", "~/README.md1");
    match res {
      Status::OkNone => return,
      _ => panic!()
    }
  }

  #[test]
  fn test_remove_file() {
    let hm = construct_hashmap();
    let syncer = RemoteSync::new(&hm);
    let res = syncer.remov_file("~/README.md1", true);
    match res {
      Status::OkNone => return,
      _ => panic!()
    }
  }
}