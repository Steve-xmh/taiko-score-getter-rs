pub async fn is_cert_installed() -> bool {
    let p = tokio::process::Command::new("security")
        .arg("find-certificate")
        .arg("-c")
        .arg("Taiko Score Getter Certificate")
        .arg("-p")
        .output()
        .await
        .expect("无法检查证书是否已安装");

    p.status.success()
}

pub async fn is_cert_trusted() -> bool {
    let config_path = crate::get_config_dir();
    let cer_path = config_path.as_path().join("ca.cer");

    let p = tokio::process::Command::new("security")
        .arg("verify-cert")
        .arg("-c")
        .arg(cer_path.as_path())
        .output()
        .await
        .expect("无法检查证书是否已信任");
    
    p.status.success()
}

pub async fn install_cert() {
    let config_path = crate::get_config_dir();
    let cer_path = config_path.as_path().join("ca.cer");

    let p = tokio::process::Command::new("osascript")
    .arg("-e")
    .arg(format!(
        r#"do shell script "security authorizationdb write com.apple.trust-settings.admin allow ; security add-trusted-cert -d -r trustAsRoot -p ssl -p basic -s \"localhost\" -k /Library/Keychains/System.keychain \"{}\" ; security authorizationdb remove com.apple.trust-settings.admin" with prompt "太鼓成绩提取器需要安装代理证书" with administrator privileges"#,
        cer_path.as_path().to_string_lossy()
    ))
    .output()
    .await
    .expect("无法安装证书");

    if !p.status.success() {
        panic!("证书安装失败");
    }
}

#[derive(Debug, Default)]
struct ProxyEntry {
    device: String,

    proxy_host: String,
    proxy_port: u16,
    proxy_state: bool,

    proxy_secure_host: String,
    proxy_secure_port: u16,
    proxy_secure_state: bool,
}

#[derive(Debug)]
pub struct ProxyConfigs {
    entries: Vec<ProxyEntry>,
}

impl ProxyConfigs {
    pub async fn new() -> Self {
        let mut entries = Vec::with_capacity(8);

        let list_hardware_ports = tokio::process::Command::new("networksetup")
            .arg("-listallhardwareports")
            .output()
            .await
            .expect("无法获取硬件端口列表");

        for line in String::from_utf8_lossy(&list_hardware_ports.stdout).lines() {
            if let Some(device_name) = line.strip_prefix("Hardware Port: ") {
                let mut entry = ProxyEntry {
                    device: device_name.to_string(),
                    ..Default::default()
                };

                let proxy_state = tokio::process::Command::new("networksetup")
                    .arg("-getwebproxy")
                    .arg(device_name)
                    .output()
                    .await
                    .expect("无法获取代理状态");
                let proxy_state = String::from_utf8_lossy(&proxy_state.stdout);

                for line in proxy_state.lines() {
                    if line == "Enabled: Yes" {
                        entry.proxy_state = true;
                    } else if line == "Enabled: No" {
                        entry.proxy_state = false;
                    } else if let Some(host) = line.strip_prefix("Server: ") {
                        entry.proxy_host = host.to_string();
                    } else if let Some(port) = line.strip_prefix("Port: ") {
                        entry.proxy_port = port.parse().expect("无法解析代理端口");
                    }
                }

                let proxy_secure_state = tokio::process::Command::new("networksetup")
                    .arg("-getsecurewebproxy")
                    .arg(device_name)
                    .output()
                    .await
                    .expect("无法获取安全代理状态");

                if !proxy_secure_state.status.success() {
                    continue;
                }

                let proxy_secure_state = String::from_utf8_lossy(&proxy_secure_state.stdout);

                for line in proxy_secure_state.lines() {
                    if line == "Enabled: Yes" {
                        entry.proxy_secure_state = true;
                    } else if line == "Enabled: No" {
                        entry.proxy_secure_state = false;
                    } else if let Some(host) = line.strip_prefix("Server: ") {
                        entry.proxy_secure_host = host.to_string();
                    } else if let Some(port) = line.strip_prefix("Port: ") {
                        entry.proxy_secure_port = port.parse().expect("无法解析安全代理端口");
                    }
                }

                tracing::debug!("发现代理配置 {:?}", entry);

                entries.push(entry);
            }
        }

        Self { entries }
    }

    pub async fn recover(&self) {
        for entry in &self.entries {
            tracing::debug!("正在还原配置 {:?}", entry);

            tokio::process::Command::new("networksetup")
                .arg("-setwebproxy")
                .arg(&entry.device)
                .arg(entry.proxy_host.clone())
                .arg(entry.proxy_port.to_string())
                .output()
                .await
                .expect("无法恢复代理");

            tokio::process::Command::new("networksetup")
                .arg("-setsecurewebproxy")
                .arg(&entry.device)
                .arg(entry.proxy_secure_host.clone())
                .arg(entry.proxy_secure_port.to_string())
                .output()
                .await
                .expect("无法恢复安全代理");

            tokio::process::Command::new("networksetup")
                .arg("-setwebproxystate")
                .arg(&entry.device)
                .arg(if entry.proxy_state { "on" } else { "off" })
                .output()
                .await
                .expect("无法恢复代理状态");

            tokio::process::Command::new("networksetup")
                .arg("-setsecurewebproxystate")
                .arg(&entry.device)
                .arg(if entry.proxy_secure_state {
                    "on"
                } else {
                    "off"
                })
                .output()
                .await
                .expect("无法恢复安全代理状态");
        }
    }

    pub async fn set_proxy(&self, proxy_host: impl AsRef<str>, proxy_port: u16) {
        let proxy_host = proxy_host.as_ref();
        for entry in &self.entries {
            tokio::process::Command::new("networksetup")
                .arg("-setwebproxy")
                .arg(&entry.device)
                .arg(proxy_host)
                .arg(proxy_port.to_string())
                .output()
                .await
                .expect("无法设置代理");

            tokio::process::Command::new("networksetup")
                .arg("-setsecurewebproxy")
                .arg(&entry.device)
                .arg(proxy_host)
                .arg(proxy_port.to_string())
                .output()
                .await
                .expect("无法设置安全代理");

            tokio::process::Command::new("networksetup")
                .arg("-setwebproxy")
                .arg(&entry.device)
                .arg(proxy_host)
                .arg(proxy_port.to_string())
                .output()
                .await
                .expect("无法设置代理");

            tokio::process::Command::new("networksetup")
                .arg("-setwebproxystate")
                .arg(&entry.device)
                .arg("on")
                .output()
                .await
                .expect("无法启用代理");

            tokio::process::Command::new("networksetup")
                .arg("-setsecurewebproxystate")
                .arg(&entry.device)
                .arg("on")
                .output()
                .await
                .expect("无法启用安全代理");
        }
    }
}
