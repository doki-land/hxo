use hxo_types::Result;

pub fn parse(source: &str) -> Result<String> {
    // 模拟 Pug 到 HTML 的转换
    let html = format!("<!-- Converted from Pug -->\n<div>{}</div>", source);
    Ok(html)
}
