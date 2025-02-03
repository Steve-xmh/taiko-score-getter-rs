use hudsucker::{certificate_authority::RcgenAuthority, rcgen::*, rustls::crypto::aws_lc_rs};

// TODO: 其他系统支持
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::ProxyConfigs;
#[cfg(target_os = "windows")]
use windows::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::ProxyConfigs;
#[cfg(target_os = "macos")]
use macos::*;

pub(super) fn generate_key_pair() -> KeyPair {
    KeyPair::generate().expect("无法生成证书密钥对")
}

pub async fn get_ca() -> RcgenAuthority {
    let config_path = crate::get_config_dir();
    let cer_path = config_path.join("ca.cer");
    let key_path = config_path.join("ca.key");

    let (cert, key_pair) = if cer_path.as_path().exists() && key_path.as_path().exists() {
        tracing::info!(
            "正在使用已有签名证书文件 {}",
            cer_path.as_path().to_string_lossy()
        );
        tracing::info!(
            "正在使用已有签名私钥文件 {}",
            key_path.as_path().to_string_lossy()
        );
        let key_data = tokio::fs::read_to_string(key_path)
            .await
            .expect("无法读取已有的私钥文件");
        let key_pair = KeyPair::from_pem(&key_data).expect("无法解析已有的私钥文件");
        let cer_data = tokio::fs::read_to_string(cer_path)
            .await
            .expect("无法读取已有的签名证书文件");

        (
            CertificateParams::from_ca_cert_pem(&cer_data)
                .expect("无法解析已有的签名文件")
                .self_signed(&key_pair)
                .expect("无法对此证书自签名"),
            key_pair,
        )
    } else {
        tracing::warn!(
            "正在生成新的密钥对到 {}",
            config_path.as_path().to_string_lossy()
        );
        let key_pair = generate_key_pair();

        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, "Taiko Score Getter Certificate");

        let mut cert_param = CertificateParams::default();
        cert_param.distinguished_name = distinguished_name;
        cert_param.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        let cert = cert_param
            .self_signed(&key_pair)
            .expect("无法生成随机自签证书");

        tokio::fs::create_dir_all(&config_path)
            .await
            .expect("无法创建配置目录");
        tokio::fs::write(&cer_path, cert.pem())
            .await
            .expect("无法保存证书文件");
        tokio::fs::write(&key_path, key_pair.serialize_pem())
            .await
            .expect("无法保存密钥对 PEM 文件");

        tracing::info!(
            " PEM 密钥对已写入至 {}",
            key_path.as_path().to_string_lossy()
        );
        tracing::info!(
            " CER 证书已写入至   {}",
            cer_path.as_path().to_string_lossy()
        );

        (cert, key_pair)
    };

    if !is_cert_installed().await {
        install_cert().await;
    }

    if !is_cert_trusted().await {
        tracing::warn!("证书已经安装但未信任，请按照提示操作");
        #[cfg(target_os = "macos")]
        {
            tracing::warn!("证书已安装成功，还有最后一步信任证书需要操作：");
            tracing::warn!(
                "  1. 打开 钥匙串访问 程序，找到 Taiko Score Getter Certificate 证书"
            );
            tracing::warn!(
                "  2. 在右上角搜索 Taiko Score Getter Certificate 证书，并双击打开搜索到的证书"
            );
            tracing::warn!("  3. 展开 信任 栏目，将 使用此证书时 下拉框配置为 完全信任");
            tracing::warn!("  详情可以参考 https://github.com/Steve-xmh/taiko-score-getter-rs/blob/main/MACOS.md");
        }
        crate::gui::send_msg_to_gui(crate::gui::GuiMessage::CertTrustNeeded);
        while !is_cert_trusted().await {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }
    }

    RcgenAuthority::new(key_pair, cert, 1000, aws_lc_rs::default_provider())
}
