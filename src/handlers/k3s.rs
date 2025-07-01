pub fn get_info() -> String {
    let mut content = String::from("=== K3s 轻量级 Kubernetes ===\n\n");
    
    content.push_str("K3s 是一个轻量级的 Kubernetes 发行版，适合边缘计算、IoT、CI/CD\n\n");
    
    // 安装
    content.push_str("1. 快速安装\n");
    content.push_str("   ```\n");
    content.push_str("   # 安装最新版本\n");
    content.push_str("   curl -sfL https://get.k3s.io | sh -\n\n");
    content.push_str("   # 安装指定版本\n");
    content.push_str("   curl -sfL https://get.k3s.io | INSTALL_K3S_VERSION=v1.27.4+k3s1 sh -\n\n");
    content.push_str("   # 安装时禁用 traefik\n");
    content.push_str("   curl -sfL https://get.k3s.io | INSTALL_K3S_EXEC=\"--disable traefik\" sh -\n");
    content.push_str("   ```\n\n");
    
    // 配置
    content.push_str("2. 基本配置\n");
    content.push_str("   ```\n");
    content.push_str("   # 查看配置\n");
    content.push_str("   cat /etc/rancher/k3s/k3s.yaml\n\n");
    content.push_str("   # 配置 kubectl\n");
    content.push_str("   mkdir ~/.kube\n");
    content.push_str("   cp /etc/rancher/k3s/k3s.yaml ~/.kube/config\n");
    content.push_str("   chmod 600 ~/.kube/config\n");
    content.push_str("   ```\n\n");
    
    // 常用命令
    content.push_str("3. 常用命令\n");
    content.push_str("   ```\n");
    content.push_str("   # 查看节点\n");
    content.push_str("   kubectl get nodes\n\n");
    content.push_str("   # 查看所有 pods\n");
    content.push_str("   kubectl get pods --all-namespaces\n\n");
    content.push_str("   # 查看服务\n");
    content.push_str("   kubectl get services --all-namespaces\n\n");
    content.push_str("   # 查看 k3s 服务状态\n");
    content.push_str("   systemctl status k3s\n\n");
    content.push_str("   # 查看日志\n");
    content.push_str("   journalctl -u k3s -f\n");
    content.push_str("   ```\n\n");
    
    // 添加节点
    content.push_str("4. 集群管理\n");
    content.push_str("   ```\n");
    content.push_str("   # 获取节点 token\n");
    content.push_str("   cat /var/lib/rancher/k3s/server/node-token\n\n");
    content.push_str("   # 添加 worker 节点\n");
    content.push_str("   curl -sfL https://get.k3s.io | K3S_URL=https://master-ip:6443 K3S_TOKEN=xxx sh -\n\n");
    content.push_str("   # 删除节点\n");
    content.push_str("   kubectl delete node node-name\n");
    content.push_str("   ```\n\n");
    
    // 应用部署
    content.push_str("5. 部署应用示例\n");
    content.push_str("   ```\n");
    content.push_str("   # 部署 nginx\n");
    content.push_str("   kubectl create deployment nginx --image=nginx\n");
    content.push_str("   kubectl expose deployment nginx --port=80 --type=NodePort\n\n");
    content.push_str("   # 查看部署\n");
    content.push_str("   kubectl get deployments\n");
    content.push_str("   kubectl get svc\n");
    content.push_str("   ```\n\n");
    
    // 卸载
    content.push_str("6. 卸载 K3s\n");
    content.push_str("   ```\n");
    content.push_str("   # 卸载 server\n");
    content.push_str("   /usr/local/bin/k3s-uninstall.sh\n\n");
    content.push_str("   # 卸载 agent\n");
    content.push_str("   /usr/local/bin/k3s-agent-uninstall.sh\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: K3s 默认包含了 containerd、Flannel、CoreDNS、Traefik 等组件。\n");
    
    content
}