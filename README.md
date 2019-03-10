# WatcherDo

A two phase trigger where a watcher is watching something (like file change, website change), and a doer will do something (like remote sync, send email) accroding to the watcher's detected events.

## Getting Start

WatcherDo is implemented by Rust. This is currently in MVP(Minium Viable Product) stage. Since Rust is a cross-platform language, this product can be used in all mainstream OS (MacOS, Linux, Unix, Windows)

### Prerequests

* If you are going to build this project from source, you should have Rust environments, which includes `rustc` and `cargo`.

### Installing

* From Source: Under watcherdo/
  ```
  $ cargo build --release
  ```
  And then modify the `config.ini` to fit your own settings.
  ```
  $ cargo run
  ```
  to start the application!

* Or you can directly get the pre-released executable program from [here](https://github.com/ralzq01/watcherdo/tree/master/release). And place the `config.ini` file in same place where the downloaded program are.


## Features

### Watcher

* FileWatcher

  FileWatcher is mainly used to detect file changes (write, create, rename, remove). In `config.ini` there will be some attributes you should specify:

  * `filepath` : specify the directory you want to watch. Should use **Absolute Path**
  * `recursive` : whether to detect folders' change recursivly.
  * `check_interval_secs`: in which interval seconds will the FileWatcher check the changed events
  * `ignore`: ignored filename or directory (**currently not support**)

### Doer

* RemoteSync

  RemoteSync is mainly used to sync the files from the local to the remote. In `config.ini` there will be some attributes you should specify:

  * `remote_ip_port`: the ssh connect ip-port, usually the port should be 22.
  * `user`: the user name
  * `password`: the password. (currently only support connect with password.)
  * `remote_dir`: the remote directory you want to sync. Should use **Absolute Path**, and should have common file directory at the end of `FileWatcher:filepath`
  * `local_dir`: not in use yet.

* SendEmail

  Under Development.

## Running the test

Since Rust `cargo` directly supports running the unit test with `#[test]`, you can directly use
```
$ cargo test
```
to run and view the test results.

## Lookup Document API

Since Rust `cargo` directly supports viewing docs in browser, you can directly use
```
$ cargo doc
```
to run and view the documents.

## Contributions

All kinds of contributions are welcomed!

When contributing to this repository, please first discuss the change you wish to make via issue, email, or any other method with the owners of this repository before making a change.

## Architecture

Watcher - Pipe (under development) - Doer

The output of the watcher will server as the input for the Doer.

