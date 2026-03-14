use base64::Engine;

#[tauri::command]
pub async fn pick_image() -> Result<String, String> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        let path = rfd::FileDialog::new()
            .add_filter("图片", &["jpg", "jpeg", "png", "bmp"])
            .pick_file()
            .ok_or_else(|| "未选择文件".to_string())?;
        let bytes = std::fs::read(path).map_err(|error| error.to_string())?;
        return Ok(base64::engine::general_purpose::STANDARD.encode(bytes));
    }

    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        Err("移动端拍照/选图接口尚未实现".to_string())
    }
}

#[tauri::command]
pub async fn take_photo() -> Result<String, String> {
    Err("当前平台暂不支持拍照".to_string())
}
