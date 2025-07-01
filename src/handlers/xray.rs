pub fn get_info() -> String {
    let mut content = String::from("=== xray 一键脚本 ===\n\n");
    
    content.push_str("Xray 是 V2Ray 的超集，具有更好的性能\n\n");
    
    // 官方安装脚本
    content.push_str("1. 官方安装脚本\n");
    content.push_str("   ```\n");
    content.push_str("   # 安装或更新 Xray\n");
    content.push_str("   bash -c \"$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)\" @ install\n\n");
    content.push_str("   # 安装指定版本\n");
    content.push_str("   bash -c \"$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)\" @ install --version 1.8.0\n\n");
    content.push_str("   # 移除 Xray\n");
    content.push_str("   bash -c \"$(curl -L https://github.com/XTLS/Xray-install/raw/main/install-release.sh)\" @ remove\n");
    content.push_str("   ```\n\n");
    
    // 配置文件
    content.push_str("2. 配置文件管理\n");
    content.push_str("   ```\n");
    content.push_str("   # 默认配置文件路径\n");
    content.push_str("   /usr/local/etc/xray/config.json\n\n");
    content.push_str("   # 编辑配置文件\n");
    content.push_str("   nano /usr/local/etc/xray/config.json\n\n");
    content.push_str("   # 验证配置文件\n");
    content.push_str("   xray run -test -config /usr/local/etc/xray/config.json\n");
    content.push_str("   ```\n\n");
    
    // 服务管理
    content.push_str("3. 服务管理命令\n");
    content.push_str("   ```\n");
    content.push_str("   # 启动 Xray\n");
    content.push_str("   systemctl start xray\n\n");
    content.push_str("   # 停止 Xray\n");
    content.push_str("   systemctl stop xray\n\n");
    content.push_str("   # 重启 Xray\n");
    content.push_str("   systemctl restart xray\n\n");
    content.push_str("   # 查看运行状态\n");
    content.push_str("   systemctl status xray\n\n");
    content.push_str("   # 设置开机自启\n");
    content.push_str("   systemctl enable xray\n\n");
    content.push_str("   # 查看日志\n");
    content.push_str("   journalctl -u xray -f\n");
    content.push_str("   ```\n\n");
    
    // 常用配置示例
    content.push_str("4. 基础配置示例\n");
    content.push_str("   ```json\n");
    content.push_str("   {\n");
    content.push_str("     \"log\": {\n");
    content.push_str("       \"loglevel\": \"warning\"\n");
    content.push_str("     },\n");
    content.push_str("     \"inbounds\": [...],\n");
    content.push_str("     \"outbounds\": [...]\n");
    content.push_str("   }\n");
    content.push_str("   ```\n\n");
    
    // 其他工具
    content.push_str("5. 相关工具\n");
    content.push_str("   ```\n");
    content.push_str("   # 生成 UUID\n");
    content.push_str("   xray uuid\n\n");
    content.push_str("   # 生成配置文件\n");
    content.push_str("   xray help config\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 请根据实际需求配置，注意防火墙规则。\n");
    
    content
}