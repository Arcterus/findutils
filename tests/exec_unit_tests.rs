// Copyright 2017 Google Inc.
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.


/// ! This file contains what would be normally be unit tests for find::matchers::exec.
/// ! But as the tests require running an external executable, they need to be run
/// ! as integration tests so we can ensure that our testing-commandline binary
/// ! has been built.
extern crate findutils;
extern crate tempdir;
extern crate walkdir;


use std::env;
use std::fs::File;
use std::io::Read;
use tempdir::TempDir;


use findutils::find::matchers::Matcher;
use findutils::find::matchers::exec::*;
use common::test_helpers::*;

mod common;

#[test]
fn matching_executes_code() {

    let temp_dir = TempDir::new("matching_executes_code").unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy();

    let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
    let matcher = SingleExecMatcher::new(&path_to_testing_commandline(),
                                         &vec![temp_dir_path.as_ref(), "abc", "{}", "xyz"],
                                         false)
        .expect("Failed to create matcher");
    let deps = FakeDependencies::new();
    assert!(matcher.matches(&abbbc, &mut deps.new_matcher_io()));

    let mut f = File::open(temp_dir.path().join("1.txt")).expect("Failed to open output file");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("failed to read output file");
    assert_eq!(s,
               format!("cwd={}\nargs=[\"abc\", \"test_data/simple/abbbc\", \"xyz\"]\n",
                       env::current_dir().unwrap().to_string_lossy()));
}

#[test]
fn matching_executes_code_in_files_directory() {

    let temp_dir = TempDir::new("matching_executes_code_in_files_directory").unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy();

    let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
    let matcher = SingleExecMatcher::new(&path_to_testing_commandline(),
                                         &vec![temp_dir_path.as_ref(), "abc", "{}", "xyz"],
                                         true)
        .expect("Failed to create matcher");
    let deps = FakeDependencies::new();
    assert!(matcher.matches(&abbbc, &mut deps.new_matcher_io()));

    let mut f = File::open(temp_dir.path().join("1.txt")).expect("Failed to open output file");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("failed to read output file");
    assert_eq!(s,
               format!("cwd={}/test_data/simple\nargs=[\"abc\", \"./abbbc\", \"xyz\"]\n",
                       env::current_dir().unwrap().to_string_lossy()));
}

#[test]
fn matching_fails_if_executable_fails() {

    let temp_dir = TempDir::new("matching_fails_if_executable_fails").unwrap();
    let temp_dir_path = temp_dir.path().to_string_lossy();

    let abbbc = get_dir_entry_for("test_data/simple", "abbbc");
    let matcher = SingleExecMatcher::new(&path_to_testing_commandline(),
                                         &vec![temp_dir_path.as_ref(),
                                               "--exit_with_failure",
                                               "abc",
                                               "{}",
                                               "xyz"],
                                         true)
        .expect("Failed to create matcher");
    let deps = FakeDependencies::new();
    assert!(!matcher.matches(&abbbc, &mut deps.new_matcher_io()));

    let mut f = File::open(temp_dir.path().join("1.txt")).expect("Failed to open output file");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("failed to read output file");
    assert_eq!(s,
               format!("cwd={}/test_data/simple\nargs=[\"--exit_with_failure\", \"abc\", \
                        \"./abbbc\", \"xyz\"]\n",
                       env::current_dir().unwrap().to_string_lossy()));
}
