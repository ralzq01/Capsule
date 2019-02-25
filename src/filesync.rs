use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

pub struct FileSync {
  dst_ip: Ipv4Addr,
  dst_file_path: Path,
  src_file_path: Path,
}

impl FileSync {
  fn new(ipv4_addr: &str, dst_file_path: Path, src_file_path: Path) -> FileSync {
    let ipv4_addr : Ipv4Addr = ipv4_addr.parse();
    FileSync {
      ipv4_addr,
      dst_file_path,
      src_file_path,
    }
  }
}