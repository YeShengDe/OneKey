pub fn get_info() -> String {
    let mut content = String::from("=== sing-box 一键脚本 ===\n\n");
    
    content.push_str("sing-box 是一个通用的代理平台\n\n");
    
    // 安装方法
    content.push_str("1. 官方安装脚本\n");
    content.push_str("   ```\n");
    content.push_str("   bash <(curl -fsSL https://sing-box.app/install.sh)\n");
    content.push_str("   ```\n\n");
    
    content.push_str("2. 手动安装\n");
    content.push_str("   ```\n");
    content.push_str("   # 下载最新版本\n");
    content.push_str("   curl -Lo sing-box.tar.gz https://github.com/SagerNet/sing-box/releases/latest/download/sing-box-linux-amd64.tar.gz\n");
    content.push_str("   tar -xzf sing-box.tar.gz\n");
    content.push_str("   cp sing-box-*/sing-box /usr/local/bin/\n");
    content.push_str("   chmod +x /usr/local/bin/sing-box\n");
    content.push_str("   ```\n\n");
    
    // 配置文件
    content.push_str("3. 配置文件位置\n");
    content.push_str("   ```\n");
    content.push_str("   # 默认配置文件路径\n");
    content.push_str("   /etc/sing-box/config.json\n\n");
    content.push_str("   # 创建配置目录\n");
    content.push_str("   mkdir -p /etc/sing-box/\n");
    content.push_str("   ```\n\n");
    
    // systemd 服务
    content.push_str("4. 系统服务管理\n");
    content.push_str("   ```\n");
    content.push_str("   # 创建服务文件\n");
    content.push_str("   cat > /etc/systemd/system/sing-box.service << EOF\n");
    content.push_str("[Unit]\n");
    content.push_str("Description=sing-box service\n");
    content.push_str("After=network.target\n\n");
    content.push_str("[Service]\n");
    content.push_str("Type=simple\n");
    content.push_str("ExecStart=/usr/local/bin/sing-box run -c /etc/sing-box/config.json\n");
    content.push_str("Restart=on-failure\n\n");
    content.push_str("[Install]\n");
    content.push_str("WantedBy=multi-user.target\n");
    content.push_str("EOF\n");
    content.push_str("   ```\n\n");
    
    // 常用命令
    content.push_str("5. 常用命令\n");
    content.push_str("   ```\n");
    content.push_str("   # 启动服务\n");
    content.push_str("   systemctl start sing-box\n\n");
    content.push_str("   # 停止服务\n");
    content.push_str("   systemctl stop sing-box\n\n");
    content.push_str("   # 重启服务\n");
    content.push_str("   systemctl restart sing-box\n\n");
    content.push_str("   # 查看状态\n");
    content.push_str("   systemctl status sing-box\n\n");
    content.push_str("   # 开机自启\n");
    content.push_str("   systemctl enable sing-box\n\n");
    content.push_str("   # 查看日志\n");
    content.push_str("   journalctl -u sing-box -f\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 配置文件需要根据实际需求编写，可参考官方文档。\n");
    
    content
}