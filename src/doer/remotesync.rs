use std::net::TcpStream;
use std::io::prelude::*;
use ssh2::Session;
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
    // connect to the remote
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
  /// e.g. input "aaa/bbb/cc.txt"
  ///      will get output of "cc.txt" 
  fn get_basename<'b> (filepath: &'b str) -> Result<&'b str, String> {
    let mut pieces = filepath.rsplit(|c| c == '/' || c == '\\');
    match pieces.next() {
      Some(p) => return Ok(p),
      None => return Err("Can't parse the filepath".to_string()),
    }
  }

  pub fn run_cmd(mut self, cmd: &str) -> Status {
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
  /// e.g. remote_path = "file/dir" or "file\dir\"
  /// e.g. the result will put the file "cc" into remote_path: "file/dir/cc"
  fn send_file(mut self, local_file_path: &str, remote_path: &str) -> Status {
    let contents = fs::read_to_string(local_file_path)
        .expect(format!("can't read from {}", local_file_path).as_str());
    let basename = RemoteSync::get_basename(local_file_path).unwrap();
    let remote_file_path = Path::new(remote_path).join(Path::new(basename));
    let lens = contents.len() as u64;
    let mut remote_file = self.sess.scp_send(&remote_file_path,
                                        0o644, lens, None).expect("create file failed");
    remote_file.write(contents.as_bytes()).expect("send file fail");
    Status::OkNone
  }

  /// Basic operations: change filename in the remote dir
  /// e.g. old_filename = "path/1/to/old.txt"
  /// e.g. new_filename = "path/2/to/new.txt"
  /// e.g. the result will be: "path/1/to/old.txt" renamed to "path/2/to/new.txt"
  fn rename_file(mut self, old_filepath: &str, new_filepath: &str) -> Status {
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
  fn remov_file(mut self, file_path: &str, is_recursive: bool) -> Status {
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
  fn get(&self, event: String) -> Status {
    println!("Remote Sync get event: {}", &event);
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
      String::from("172.23.234.88:22")
    );
    hm.insert(
      String::from("user"),
      String::from("v-zhilin")
    );
    hm.insert(
      String::from("password"),
      String::from("v-zhilin")
    );
    hm.insert(
      String::from("remote_dir"),
      String::from("~/")
    );
    hm.insert(
      String::from("local_dir"),
      String::from("./")
    );
    hm
  }

  #[test]
  fn test_password_connect() {
    let tcp_conn = TcpStream::connect("172.23.234.88:22").unwrap();
    let mut sess = Session::new().unwrap();
    sess.handshake(&tcp_conn).unwrap();
    sess.userauth_password("v-zhilin", "v-zhilin").unwrap();
    assert!(sess.authenticated());
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