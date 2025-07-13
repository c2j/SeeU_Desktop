use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use keyring::Entry;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 加密后的数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Base64编码的加密数据
    pub data: String,
    /// Base64编码的随机数
    pub nonce: String,
}

/// 密码加密管理器
pub struct PasswordEncryption {
    cipher: Aes256Gcm,
    service_name: String,
}

impl std::fmt::Debug for PasswordEncryption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PasswordEncryption")
            .field("service_name", &self.service_name)
            .field("cipher", &"<encrypted>")
            .finish()
    }
}

impl PasswordEncryption {
    /// 创建新的密码加密管理器
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let service_name = "seeu_desktop_iterminal".to_string();
        let key = Self::get_or_create_key(&service_name)?;
        let cipher = Aes256Gcm::new(&key);

        Ok(Self {
            cipher,
            service_name,
        })
    }

    /// 加密字符串
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        if plaintext.is_empty() {
            return Ok(EncryptedData {
                data: String::new(),
                nonce: String::new(),
            });
        }

        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| format!("加密失败: {}", e))?;

        Ok(EncryptedData {
            data: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(&nonce),
        })
    }

    /// 解密字符串
    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<String, Box<dyn std::error::Error>> {
        if encrypted.data.is_empty() && encrypted.nonce.is_empty() {
            return Ok(String::new());
        }

        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted.data)
            .map_err(|e| format!("解码密文失败: {}", e))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| format!("解码随机数失败: {}", e))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("解密失败: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| format!("UTF-8转换失败: {}", e).into())
    }

    /// 批量加密字符串映射
    pub fn encrypt_map(
        &self,
        data: &HashMap<String, String>,
    ) -> Result<HashMap<String, EncryptedData>, Box<dyn std::error::Error>> {
        let mut encrypted_map = HashMap::new();

        for (key, value) in data {
            encrypted_map.insert(key.clone(), self.encrypt(value)?);
        }

        Ok(encrypted_map)
    }

    /// 批量解密字符串映射
    pub fn decrypt_map(
        &self,
        encrypted_data: &HashMap<String, EncryptedData>,
    ) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let mut decrypted_map = HashMap::new();

        for (key, encrypted) in encrypted_data {
            decrypted_map.insert(key.clone(), self.decrypt(encrypted)?);
        }

        Ok(decrypted_map)
    }

    /// 获取或创建加密密钥
    fn get_or_create_key(service_name: &str) -> Result<Key<Aes256Gcm>, Box<dyn std::error::Error>> {
        let entry = Entry::new(service_name, "encryption_key")?;

        // 尝试从系统密钥环获取现有密钥
        match entry.get_password() {
            Ok(key_str) => {
                // 解码现有密钥
                let key_bytes = general_purpose::STANDARD
                    .decode(&key_str)
                    .map_err(|e| format!("解码密钥失败: {}", e))?;

                if key_bytes.len() == 32 {
                    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
                    log::info!("从系统密钥环加载现有加密密钥");
                    return Ok(*key);
                } else {
                    log::warn!("系统密钥环中的密钥长度不正确，将生成新密钥");
                }
            }
            Err(keyring::Error::NoEntry) => {
                log::info!("系统密钥环中未找到密钥，将生成新密钥");
            }
            Err(e) => {
                log::warn!("访问系统密钥环失败: {}，将生成新密钥", e);
            }
        }

        // 生成新的随机密钥
        let mut key_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut key_bytes);
        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);

        // 尝试保存到系统密钥环
        let key_str = general_purpose::STANDARD.encode(&key_bytes);
        match entry.set_password(&key_str) {
            Ok(_) => {
                log::info!("新加密密钥已保存到系统密钥环");
            }
            Err(keyring::Error::PlatformFailure(ref platform_error)) => {
                let error_msg = format!("{:?}", platform_error);
                if error_msg.contains("already exists") {
                    // 密钥已存在，尝试更新
                    log::info!("密钥已存在于系统密钥环中");
                    // 对于已存在的情况，我们可以继续使用现有密钥
                } else {
                    log::warn!("平台密钥环错误: {}，密钥将仅在内存中使用", error_msg);
                }
            }
            Err(e) => {
                log::warn!("无法将密钥保存到系统密钥环: {}，密钥将仅在内存中使用", e);
            }
        }

        Ok(*key)
    }

    /// 测试加密和解密功能
    pub fn test_encryption(&self) -> Result<(), Box<dyn std::error::Error>> {
        let test_data = "这是一个测试密码123!@#";
        let encrypted = self.encrypt(test_data)?;
        let decrypted = self.decrypt(&encrypted)?;

        if test_data == decrypted {
            log::info!("加密模块测试通过");
            Ok(())
        } else {
            Err("加密模块测试失败：解密后的数据不匹配".into())
        }
    }
}

/// 用于序列化的加密数据容器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedContainer {
    /// 加密的敏感数据
    pub encrypted_fields: HashMap<String, EncryptedData>,
    /// 非敏感的明文数据
    pub plain_fields: HashMap<String, serde_json::Value>,
    /// 加密版本，用于未来的兼容性
    pub encryption_version: u32,
}

impl EncryptedContainer {
    /// 创建新的加密容器
    pub fn new() -> Self {
        Self {
            encrypted_fields: HashMap::new(),
            plain_fields: HashMap::new(),
            encryption_version: 1,
        }
    }

    /// 添加加密字段
    pub fn add_encrypted_field(&mut self, key: String, encrypted_data: EncryptedData) {
        self.encrypted_fields.insert(key, encrypted_data);
    }

    /// 添加明文字段
    pub fn add_plain_field(&mut self, key: String, value: serde_json::Value) {
        self.plain_fields.insert(key, value);
    }

    /// 获取加密字段
    pub fn get_encrypted_field(&self, key: &str) -> Option<&EncryptedData> {
        self.encrypted_fields.get(key)
    }

    /// 获取明文字段
    pub fn get_plain_field(&self, key: &str) -> Option<&serde_json::Value> {
        self.plain_fields.get(key)
    }
}

impl Default for EncryptedContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let encryption = PasswordEncryption::new().expect("Failed to create encryption");
        
        let original = "test_password_123";
        let encrypted = encryption.encrypt(original).expect("Failed to encrypt");
        let decrypted = encryption.decrypt(&encrypted).expect("Failed to decrypt");
        
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_string_encryption() {
        let encryption = PasswordEncryption::new().expect("Failed to create encryption");
        
        let encrypted = encryption.encrypt("").expect("Failed to encrypt empty string");
        let decrypted = encryption.decrypt(&encrypted).expect("Failed to decrypt empty string");
        
        assert_eq!("", decrypted);
        assert_eq!("", encrypted.data);
        assert_eq!("", encrypted.nonce);
    }

    #[test]
    fn test_map_encryption() {
        let encryption = PasswordEncryption::new().expect("Failed to create encryption");
        
        let mut data = HashMap::new();
        data.insert("password".to_string(), "secret123".to_string());
        data.insert("passphrase".to_string(), "another_secret".to_string());
        
        let encrypted_map = encryption.encrypt_map(&data).expect("Failed to encrypt map");
        let decrypted_map = encryption.decrypt_map(&encrypted_map).expect("Failed to decrypt map");
        
        assert_eq!(data, decrypted_map);
    }

    #[test]
    fn test_encrypted_container() {
        let mut container = EncryptedContainer::new();
        
        let encryption = PasswordEncryption::new().expect("Failed to create encryption");
        let encrypted_data = encryption.encrypt("secret").expect("Failed to encrypt");
        
        container.add_encrypted_field("password".to_string(), encrypted_data);
        container.add_plain_field("username".to_string(), serde_json::Value::String("user".to_string()));
        
        assert!(container.get_encrypted_field("password").is_some());
        assert!(container.get_plain_field("username").is_some());
        assert_eq!(container.encryption_version, 1);
    }
}
