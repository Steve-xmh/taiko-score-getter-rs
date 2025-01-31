use sysproxy::Sysproxy;

pub async fn is_cert_installed() -> bool {
    let p = tokio::process::Command::new("certutil")
        .arg("-verifystore")
        .arg("root")
        .arg("Taiko Score Getter Certificate")
        .output()
        .await
        .expect("无法检查证书是否已安装");

    p.status.success()
}

pub async fn install_cert() {
    let config_path = crate::get_config_dir();
    let cer_path = config_path.as_path().join("ca.cer");

    // certutil.exe -addstore root mitmproxy-ca-cert.cer
    tokio::process::Command::new("certutil")
        .arg("-addstore")
        .arg("root")
        .arg(cer_path)
        .status()
        .await
        .expect("无法安装证书");

    tracing::info!("证书已安装");
}

#[derive(Debug)]
pub struct ProxyConfigs {
    last_proxy: Sysproxy,
}

impl ProxyConfigs {
    pub async fn new() -> Self {
        Self {
            last_proxy: Sysproxy::get_system_proxy().expect("无法获取系统代理配置"),
        }
    }

    pub async fn recover(&self) {
        self.last_proxy
            .set_system_proxy()
            .expect("无法还原系统代理配置");
    }

    pub async fn set_proxy(&self, proxy_host: impl AsRef<str>, proxy_port: u16) {
        let mut proxy = self.last_proxy.clone();
        proxy.enable = true;
        proxy.host = proxy_host.as_ref().to_string();
        proxy.port = proxy_port;

        proxy.set_system_proxy().expect("无法设置系统代理配置");
    }
}
