use std::process::Command;
use std::io;

pub struct CommandRunner;

impl CommandRunner {
    /// 执行系统命令并返回输出
    pub fn run(cmd: &str, args: &[&str]) -> io::Result<String> {
        let output = Command::new(cmd)
            .args(args)
            .output()?;
        
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                String::from_utf8_lossy(&output.stderr),
            ))
        }
    }
    
    /// 执行命令并返回是否成功
    pub fn run_status(cmd: &str, args: &[&str]) -> io::Result<bool> {
        let status = Command::new(cmd)
            .args(args)
            .status()?;
        
        Ok(status.success())
    }
    
    /// 执行需要 sudo 权限的命令
    pub fn run_sudo(cmd: &str, args: &[&str]) -> io::Result<String> {
        let mut sudo_args = vec![cmd];
        sudo_args.extend_from_slice(args);
        
        Self::run("sudo", &sudo_args)
    }
    
    /// 检查命令是否存在
    pub fn command_exists(cmd: &str) -> bool {
        Command::new("which")
            .arg(cmd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_command_exists() {
        // 大多数 Linux 系统都有 ls 命令
        assert!(CommandRunner::command_exists("ls"));
        
        // 不太可能存在的命令
        assert!(!CommandRunner::command_exists("this_command_does_not_exist"));
    }
}