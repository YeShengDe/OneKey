pub fn get_info() -> String {
    let mut content = String::from("=== Kubernetes (K8s) ===\n\n");
    
    content.push_str("Kubernetes 是容器编排的行业标准\n\n");
    
    // 前置要求
    content.push_str("1. 前置要求\n");
    content.push_str("   ```\n");
    content.push_str("   # 关闭 swap\n");
    content.push_str("   swapoff -a\n");
    content.push_str("   sed -i '/ swap / s/^/#/' /etc/fstab\n\n");
    content.push_str("   # 配置内核参数\n");
    content.push_str("   cat <<EOF | tee /etc/modules-load.d/k8s.conf\n");
    content.push_str("br_netfilter\n");
    content.push_str("EOF\n\n");
    content.push_str("   cat <<EOF | tee /etc/sysctl.d/k8s.conf\n");
    content.push_str("net.bridge.bridge-nf-call-ip6tables = 1\n");
    content.push_str("net.bridge.bridge-nf-call-iptables = 1\n");
    content.push_str("EOF\n");
    content.push_str("   sysctl --system\n");
    content.push_str("   ```\n\n");
    
    // 安装容器运行时
    content.push_str("2. 安装容器运行时 (containerd)\n");
    content.push_str("   ```\n");
    content.push_str("   # 安装 containerd\n");
    content.push_str("   apt-get update\n");
    content.push_str("   apt-get install -y containerd\n\n");
    content.push_str("   # 配置 containerd\n");
    content.push_str("   mkdir -p /etc/containerd\n");
    content.push_str("   containerd config default | tee /etc/containerd/config.toml\n");
    content.push_str("   systemctl restart containerd\n");
    content.push_str("   ```\n\n");
    
    // 安装 kubeadm
    content.push_str("3. 安装 kubeadm, kubelet, kubectl\n");
    content.push_str("   ```\n");
    content.push_str("   # 添加 Kubernetes APT 仓库\n");
    content.push_str("   curl -fsSL https://packages.cloud.google.com/apt/doc/apt-key.gpg | apt-key add -\n");
    content.push_str("   echo \"deb https://apt.kubernetes.io/ kubernetes-xenial main\" > /etc/apt/sources.list.d/kubernetes.list\n\n");
    content.push_str("   # 安装组件\n");
    content.push_str("   apt-get update\n");
    content.push_str("   apt-get install -y kubelet kubeadm kubectl\n");
    content.push_str("   apt-mark hold kubelet kubeadm kubectl\n");
    content.push_str("   ```\n\n");
    
    // 初始化集群
    content.push_str("4. 初始化 Master 节点\n");
    content.push_str("   ```\n");
    content.push_str("   # 初始化集群\n");
    content.push_str("   kubeadm init --pod-network-cidr=10.244.0.0/16\n\n");
    content.push_str("   # 配置 kubectl\n");
    content.push_str("   mkdir -p $HOME/.kube\n");
    content.push_str("   cp -i /etc/kubernetes/admin.conf $HOME/.kube/config\n");
    content.push_str("   chown $(id -u):$(id -g) $HOME/.kube/config\n");
    content.push_str("   ```\n\n");
    
    // 网络插件
    content.push_str("5. 安装网络插件\n");
    content.push_str("   ```\n");
    content.push_str("   # 安装 Flannel\n");
    content.push_str("   kubectl apply -f https://raw.githubusercontent.com/flannel-io/flannel/master/Documentation/kube-flannel.yml\n\n");
    content.push_str("   # 或安装 Calico\n");
    content.push_str("   kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.26.0/manifests/tigera-operator.yaml\n");
    content.push_str("   kubectl create -f https://raw.githubusercontent.com/projectcalico/calico/v3.26.0/manifests/custom-resources.yaml\n");
    content.push_str("   ```\n\n");
    
    // 加入节点
    content.push_str("6. 添加 Worker 节点\n");
    content.push_str("   ```\n");
    content.push_str("   # 在 Master 上生成加入命令\n");
    content.push_str("   kubeadm token create --print-join-command\n\n");
    content.push_str("   # 在 Worker 节点上执行加入命令\n");
    content.push_str("   kubeadm join master-ip:6443 --token xxx --discovery-token-ca-cert-hash sha256:xxx\n");
    content.push_str("   ```\n\n");
    
    // 常用命令
    content.push_str("7. 常用管理命令\n");
    content.push_str("   ```\n");
    content.push_str("   # 查看集群信息\n");
    content.push_str("   kubectl cluster-info\n");
    content.push_str("   kubectl get nodes\n");
    content.push_str("   kubectl get pods -A\n\n");
    content.push_str("   # 部署应用\n");
    content.push_str("   kubectl create deployment nginx --image=nginx\n");
    content.push_str("   kubectl expose deployment nginx --port=80 --type=LoadBalancer\n");
    content.push_str("   ```\n\n");
    
    content.push_str("提示: 生产环境建议使用高可用部署，至少3个Master节点。\n");
    
    content
}