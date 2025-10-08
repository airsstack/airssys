//! Operation builders for framework ergonomic APIs.

use std::time::Duration;

use super::OSLFramework;
use crate::core::{executor::ExecutionResult, result::OSResult};
use crate::operations::{
    filesystem::{
        DirectoryCreateOperation, DirectoryListOperation, FileDeleteOperation, FileReadOperation,
        FileWriteOperation,
    },
    network::{NetworkConnectOperation, NetworkListenOperation, NetworkSocketOperation},
    process::{ProcessKillOperation, ProcessSignalOperation, ProcessSpawnOperation},
};

/// Builder for filesystem operations.
pub struct FilesystemBuilder<'a> {
    framework: &'a OSLFramework,
    timeout: Option<Duration>,
}

impl<'a> FilesystemBuilder<'a> {
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self {
            framework,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn read_file(self, path: &str) -> FileReadOperationWrapper<'a> {
        FileReadOperationWrapper {
            framework: self.framework,
            path: path.to_string(),
            timeout: self.timeout,
        }
    }

    pub fn write_file(self, path: &str) -> FileWriteOperationWrapper<'a> {
        FileWriteOperationWrapper {
            framework: self.framework,
            path: path.to_string(),
            timeout: self.timeout,
            content: Vec::new(),
        }
    }

    pub fn create_directory(self, path: &str) -> DirectoryCreateOperationWrapper<'a> {
        DirectoryCreateOperationWrapper {
            framework: self.framework,
            path: path.to_string(),
            timeout: self.timeout,
            recursive: false,
        }
    }

    pub fn list_directory(self, path: &str) -> DirectoryListOperationWrapper<'a> {
        DirectoryListOperationWrapper {
            framework: self.framework,
            path: path.to_string(),
            timeout: self.timeout,
        }
    }

    pub fn delete_file(self, path: &str) -> FileDeleteOperationWrapper<'a> {
        FileDeleteOperationWrapper {
            framework: self.framework,
            path: path.to_string(),
            timeout: self.timeout,
        }
    }
}

// Filesystem operation wrappers

#[allow(dead_code)]
pub struct FileReadOperationWrapper<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
}

impl<'a> FileReadOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = FileReadOperation::new(&self.path);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct FileWriteOperationWrapper<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
    content: Vec<u8>,
}

impl<'a> FileWriteOperationWrapper<'a> {
    pub fn with_content(mut self, content: Vec<u8>) -> Self {
        self.content = content;
        self
    }

    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = FileWriteOperation::new(&self.path, self.content);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct DirectoryCreateOperationWrapper<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
    recursive: bool,
}

impl<'a> DirectoryCreateOperationWrapper<'a> {
    pub fn recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let mut operation = DirectoryCreateOperation::new(&self.path);
        if self.recursive {
            operation = operation.recursive();
        }
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct DirectoryListOperationWrapper<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
}

impl<'a> DirectoryListOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = DirectoryListOperation::new(&self.path);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct FileDeleteOperationWrapper<'a> {
    framework: &'a OSLFramework,
    path: String,
    timeout: Option<Duration>,
}

impl<'a> FileDeleteOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = FileDeleteOperation::new(&self.path);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}


/// Builder for process operations.
pub struct ProcessBuilder<'a> {
    framework: &'a OSLFramework,
    timeout: Option<Duration>,
}

impl<'a> ProcessBuilder<'a> {
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self {
            framework,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn spawn(self, command: &str) -> ProcessSpawnOperationWrapper<'a> {
        ProcessSpawnOperationWrapper {
            framework: self.framework,
            command: command.to_string(),
            timeout: self.timeout,
            args: Vec::new(),
            working_dir: None,
        }
    }

    pub fn kill(self, pid: u32) -> ProcessKillOperationWrapper<'a> {
        ProcessKillOperationWrapper {
            framework: self.framework,
            pid,
            timeout: self.timeout,
        }
    }

    pub fn signal(self, pid: u32, signal: i32) -> ProcessSignalOperationWrapper<'a> {
        ProcessSignalOperationWrapper {
            framework: self.framework,
            pid,
            signal,
            timeout: self.timeout,
        }
    }
}

// Process operation wrappers

#[allow(dead_code)]
pub struct ProcessSpawnOperationWrapper<'a> {
    framework: &'a OSLFramework,
    command: String,
    timeout: Option<Duration>,
    args: Vec<String>,
    working_dir: Option<String>,
}

impl<'a> ProcessSpawnOperationWrapper<'a> {
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    pub fn with_working_dir(mut self, working_dir: String) -> Self {
        self.working_dir = Some(working_dir);
        self
    }

    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let mut operation = ProcessSpawnOperation::new(&self.command);
        if !self.args.is_empty() {
            operation = operation.with_args(self.args);
        }
        if let Some(working_dir) = self.working_dir {
            operation = operation.working_dir(working_dir);
        }
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct ProcessKillOperationWrapper<'a> {
    framework: &'a OSLFramework,
    pid: u32,
    timeout: Option<Duration>,
}

impl<'a> ProcessKillOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = ProcessKillOperation::new(self.pid);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct ProcessSignalOperationWrapper<'a> {
    framework: &'a OSLFramework,
    pid: u32,
    signal: i32,
    timeout: Option<Duration>,
}

impl<'a> ProcessSignalOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = ProcessSignalOperation::new(self.pid, self.signal);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}



/// Builder for network operations.
pub struct NetworkBuilder<'a> {
    framework: &'a OSLFramework,
    timeout: Option<Duration>,
}

impl<'a> NetworkBuilder<'a> {
    pub(crate) fn new(framework: &'a OSLFramework) -> Self {
        Self {
            framework,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn connect(self, address: &str) -> NetworkConnectOperationWrapper<'a> {
        NetworkConnectOperationWrapper {
            framework: self.framework,
            address: address.to_string(),
            timeout: self.timeout,
        }
    }

    pub fn listen(self, address: &str) -> NetworkListenOperationWrapper<'a> {
        NetworkListenOperationWrapper {
            framework: self.framework,
            address: address.to_string(),
            timeout: self.timeout,
            backlog: None,
            socket_path: None,
        }
    }

    pub fn create_socket(self, socket_type: &str) -> NetworkSocketOperationWrapper<'a> {
        NetworkSocketOperationWrapper {
            framework: self.framework,
            socket_type: socket_type.to_string(),
            timeout: self.timeout,
        }
    }
}

// Network operation wrappers

#[allow(dead_code)]
pub struct NetworkConnectOperationWrapper<'a> {
    framework: &'a OSLFramework,
    address: String,
    timeout: Option<Duration>,
}

impl<'a> NetworkConnectOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = NetworkConnectOperation::new(&self.address);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct NetworkListenOperationWrapper<'a> {
    framework: &'a OSLFramework,
    address: String,
    timeout: Option<Duration>,
    backlog: Option<i32>,
    socket_path: Option<String>,
}

impl<'a> NetworkListenOperationWrapper<'a> {
    pub fn with_backlog(mut self, backlog: i32) -> Self {
        self.backlog = Some(backlog);
        self
    }

    pub fn with_socket_path(mut self, socket_path: String) -> Self {
        self.socket_path = Some(socket_path);
        self
    }

    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let mut operation = NetworkListenOperation::new(&self.address);
        if let Some(backlog) = self.backlog {
            operation = operation.with_backlog(backlog);
        }
        if let Some(socket_path) = self.socket_path {
            operation = operation.with_socket_path(socket_path);
        }
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}

#[allow(dead_code)]
pub struct NetworkSocketOperationWrapper<'a> {
    framework: &'a OSLFramework,
    socket_type: String,
    timeout: Option<Duration>,
}

impl<'a> NetworkSocketOperationWrapper<'a> {
    pub async fn execute(self) -> OSResult<ExecutionResult> {
        let operation = NetworkSocketOperation::new(&self.socket_type);
        // TODO: Apply timeout when Operation trait supports it
        self.framework.execute(operation).await
    }
}
