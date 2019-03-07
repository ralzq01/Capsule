use std::path::Path;
use std::net::TcpStream;
use ssh2::Session;
use std::collections::HashMap;

pub struct RemoteSync<'a> {
  ip_port: &'a str,
  user: &'a str,
  password: &'a str,
  remote_dir: &'a Path,
  local_dir: &'a Path,
  tcp_conn: TcpStream,
  sess: mut Session,
}

impl RemoteSync<'a> {
  fn new(config: &'a HashMap<String, String>) -> FileSync {
    let ip_port = ip: config.get("remote_ip_port").unwrap();
    let user = config.get("user").unwrap();
    let password = config.get("password").unwrap();
    let tcp_conn = TcpStream::connect(&ip_port);
    let mut sess = Session::new().unwrap();
    FileSync {
      ip_port: ip_port,
      user: user,
      password: password,
      remote_dir: config.get("remote_dir").unwrap(),
      local_dir: config.get("local_dir").unwrap(),
      tcp_conn: tcp_conn,
      sess: sess,
    }
  }

  /// connect to the remote device
  /// should be called befor every sync operation
  fn connect(self) {
    self.sess.handshake(&self.tcp).unwrap();
    self.sess.userauth_password(self.user, self.password);
    assert!(self.sess.authenticated());
  }

  /// Transfer file_name from local to dst dir
  fn update(file_name: String) {
    
  }
}