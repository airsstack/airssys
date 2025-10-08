//! Integration tests for filesystem operations.
//!
//! These tests validate cross-cutting behavior across all filesystem operations
//! to ensure consistency and compliance with the Operation trait requirements.

use airssys_osl::operations::{
    DirectoryCreateOperation, DirectoryListOperation, FileDeleteOperation, FileReadOperation,
    FileWriteOperation,
};

/// Test that all filesystem operations are cloneable (required by Operation trait)
#[test]
fn test_filesystem_operations_are_cloneable() {
    let file_read = FileReadOperation::new("/tmp/test.txt");
    let _cloned = file_read.clone();

    let file_write = FileWriteOperation::new("/tmp/test.txt", vec![1, 2, 3]);
    let _cloned = file_write.clone();

    let dir_create = DirectoryCreateOperation::new("/tmp/dir");
    let _cloned = dir_create.clone();

    let dir_list = DirectoryListOperation::new("/tmp");
    let _cloned = dir_list.clone();

    let file_delete = FileDeleteOperation::new("/tmp/test.txt");
    let _cloned = file_delete.clone();
}

/// Test Display implementations for all filesystem operations
#[test]
fn test_filesystem_operations_display() {
    let file_read = FileReadOperation::new("/tmp/test.txt");
    assert_eq!(format!("{file_read}"), "FileRead(/tmp/test.txt)");

    let file_write = FileWriteOperation::new("/tmp/test.txt", b"foo".to_vec());
    assert_eq!(
        format!("{file_write}"),
        "FileWrite(/tmp/test.txt, mode=write, 3 bytes)"
    );

    let dir_create = DirectoryCreateOperation::new("/tmp/dir").recursive();
    assert_eq!(
        format!("{dir_create}"),
        "DirectoryCreate(/tmp/dir, mode=recursive)"
    );

    let dir_list = DirectoryListOperation::new("/tmp");
    assert_eq!(format!("{dir_list}"), "DirectoryList(/tmp)");

    let file_delete = FileDeleteOperation::new("/tmp/test.txt");
    assert_eq!(format!("{file_delete}"), "FileDelete(/tmp/test.txt)");
}
