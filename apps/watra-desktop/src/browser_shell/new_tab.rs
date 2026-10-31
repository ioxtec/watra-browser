const HTML: &str = r#"<!doctype html>
<html>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width">
<title>New Tab</title>
<style>
* { box-sizing: border-box }
body {
  margin: 0; min-height: 100vh; display: grid; place-items: center;
  background: #202328; color: #e6e7e9;
  font: 15px -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
}
.home { margin-top: -8vh; text-align: center }
.fire { position: relative; width: 100px; height: 100px; margin: 0 auto 22px }
.glow {
  position: absolute; inset: 8px; border-radius: 50%;
  background: radial-gradient(circle, #e5a95455, transparent 70%); filter: blur(8px);
}
.flame {
  position: absolute; left: 28px; top: 7px; width: 44px; height: 63px;
  transform: rotate(45deg); border-radius: 60% 10% 55% 50%;
  background: linear-gradient(155deg, #ffd27c, #ef7848 68%, #b94a38);
  box-shadow: 0 0 26px #ef784855;
}
.core {
  position: absolute; left: 40px; top: 39px; width: 22px; height: 38px;
  transform: rotate(45deg); border-radius: 60% 20% 55% 50%;
  background: linear-gradient(#fff0b2, #f1a955);
}
h1 { margin: 0 0 8px; font-size: 23px; font-weight: 500 }
p { margin: 0 0 28px; color: #9da5b0 }
.links { display: flex; justify-content: center; gap: 12px }
.links a {
  min-width: 112px; padding: 14px 20px; border: 1px solid #383d45;
  border-radius: 14px; background: #292d33; color: #d8dce2;
  text-decoration: none; transition: background .15s;
}
.links a:hover { background: #343a43 }
.symbol { display: block; margin-bottom: 7px; font-size: 19px }
</style>
<body>
  <main class="home">
    <div class="fire" aria-hidden="true"><div class="glow"></div><div class="flame"></div><div class="core"></div></div>
    <h1>Watra</h1>
    <p>A quiet place to start browsing</p>
    <nav class="links" aria-label="Shortcuts">
      <a href="https://www.google.com"><span class="symbol">⌕</span>Google</a>
      <a href="https://www.youtube.com"><span class="symbol">▶</span>YouTube</a>
      <a href="https://www.wikipedia.org"><span class="symbol">◈</span>Wikipedia</a>
    </nav>
  </main>
</body>
</html>"#;

pub(super) fn url() -> String {
    let mut url = String::from("data:text/html;charset=utf-8,");
    for byte in HTML.bytes() {
        if byte.is_ascii_alphanumeric() || b"-_.~".contains(&byte) {
            url.push(byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(url, "%{byte:02X}");
        }
    }
    url
}

#[cfg(test)]
mod tests {
    use super::url;

    #[test]
    fn embeds_the_new_tab_page_as_a_safe_data_url() {
        let page_url = url();

        assert!(page_url.starts_with("data:text/html;charset=utf-8,%3C%21doctype%20html%3E"));
        assert!(page_url.contains("https%3A%2F%2Fwww.google.com"));
        assert!(page_url.contains("https%3A%2F%2Fwww.youtube.com"));
        assert!(page_url.contains("https%3A%2F%2Fwww.wikipedia.org"));
    }
}
