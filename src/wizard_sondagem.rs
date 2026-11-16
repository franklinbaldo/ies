//! Sondagem e inspeção da instalação remota do SEI para o assistente de configuração.

pub(crate) fn normalizar_origem(url: &str) -> Result<String, String> {
    let mut u = url.trim().to_string();
    if !u.starts_with("http://") && !u.starts_with("https://") {
        u = format!("https://{u}");
    }
    if let Some(pos) = u.find("://") {
        if !u[pos + 3..].contains('/') {
            u.push('/');
        }
    }
    let parsed = crate::url_observada::UrlHttps::interpretar(&u)
        .ok_or_else(|| format!("URL inválida: '{url}'. Deve ser HTTPS."))?;

    let host = parsed.host;
    let mut caminho = parsed.caminho.to_string();

    for entry in &[
        "sip/login.php",
        "sei/controlador.php",
        "sei/inicializar.php",
    ] {
        if let Some(pos) = caminho.to_lowercase().find(entry) {
            caminho.truncate(pos);
            break;
        }
    }
    let p = caminho.trim_matches('/');
    if p.ends_with("sei") {
        let prefixo = p.strip_suffix("sei").unwrap().trim_end_matches('/');
        if prefixo.is_empty() {
            Ok(format!("https://{host}/sei"))
        } else {
            Ok(format!("https://{host}/{prefixo}/sei"))
        }
    } else if p.is_empty() {
        Ok(format!("https://{host}/sei"))
    } else {
        Ok(format!("https://{host}/{p}/sei"))
    }
}

pub(crate) fn extrair_orgaos_do_html(html: &str) -> Vec<(String, String)> {
    let dom = match tl::parse(html, tl::ParserOptions::default()) {
        Ok(dom) => dom,
        Err(_) => return Vec::new(),
    };
    let parser = dom.parser();
    let mut orgaos = Vec::new();

    for handle in dom.nodes() {
        if let Some(tag) = handle.as_tag() {
            if tag.name() == "select" {
                let name = crate::html_atributos::valor_do_atributo(tag, "name")
                    .ok()
                    .flatten();
                let id = crate::html_atributos::valor_do_atributo(tag, "id")
                    .ok()
                    .flatten();
                let eh_sel_orgao = name
                    .as_deref()
                    .is_some_and(|n| n.eq_ignore_ascii_case("selOrgao"))
                    || id
                        .as_deref()
                        .is_some_and(|i| i.eq_ignore_ascii_case("selOrgao"));

                if eh_sel_orgao {
                    for opt_h in tag.query_selector(parser, "option").into_iter().flatten() {
                        if let Some(opt_tag) = opt_h.get(parser).and_then(tl::Node::as_tag) {
                            let valor = crate::html_atributos::valor_do_atributo(opt_tag, "value")
                                .ok()
                                .flatten()
                                .unwrap_or_default();
                            let texto = crate::transporte::texto_de_recusa::decodificar_entidades(
                                &opt_tag.inner_text(parser),
                            )
                            .split_whitespace()
                            .collect::<Vec<_>>()
                            .join(" ");

                            if !valor.is_empty() && valor != "null" && !texto.is_empty() {
                                orgaos.push((valor, texto));
                            }
                        }
                    }
                    break;
                }
            }
        }
    }
    orgaos
}

pub(crate) fn sondar_instalacao(
    origem: &str,
) -> Result<(Option<String>, Vec<(String, String)>), String> {
    let mut url_atual = origem.to_string();
    let host_esperado = match crate::url_observada::UrlHttps::interpretar(origem) {
        Some(u) => u.host.to_ascii_lowercase(),
        None => return Err("URL inválida para sondagem".to_string()),
    };

    let mut sigla_sip = None;
    let mut html_final = String::new();

    for _ in 0..6 {
        let resp = crate::transporte::cliente_https::requisitar("GET", &url_atual, "", None)
            .map_err(|e| format!("Falha de conexão ({})", e.subcodigo()))?;

        if (300..400).contains(&resp.status) {
            let loc = resp
                .cabecalho("location")
                .ok_or_else(|| "Redirecionamento sem cabeçalho Location".to_string())?;

            let proxima = if loc.starts_with('/') {
                format!("https://{host_esperado}{loc}")
            } else if let Some(u) = crate::url_observada::UrlHttps::interpretar(loc) {
                if !u.host.eq_ignore_ascii_case(&host_esperado) {
                    return Err(format!(
                        "Redirecionamento recusado para host externo ({})",
                        u.host
                    ));
                }
                loc.to_string()
            } else {
                return Err("Redirecionamento não-HTTPS recusado".to_string());
            };

            if let Some(pos) = proxima.find("sigla_orgao_sistema=") {
                let pedaco = &proxima[pos + 20..];
                let sigla = pedaco.split('&').next().unwrap_or(pedaco);
                sigla_sip = Some(sigla.to_string());
            }
            url_atual = proxima;
            continue;
        }

        if resp.status == 200 {
            use std::io::Read;
            let mut buf = Vec::new();
            let mut corpo = resp.corpo;
            let _ = corpo.read_to_end(&mut buf);
            html_final = String::from_utf8_lossy(&buf).into_owned();
            break;
        }
        return Err(format!(
            "Servidor respondeu com status HTTP {}",
            resp.status
        ));
    }

    if html_final.is_empty() {
        return Err("Nenhum conteúdo recebido da tela de login".to_string());
    }

    let orgaos = extrair_orgaos_do_html(&html_final);
    Ok((sigla_sip, orgaos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normaliza_origem_rejeita_esquema_invalido() {
        assert!(normalizar_origem("http://inseguro.gov.br").is_err());
    }

    #[test]
    fn normaliza_origem_variantes_sei() {
        assert_eq!(
            normalizar_origem("https://sei.sistemas.ro.gov.br").unwrap(),
            "https://sei.sistemas.ro.gov.br/sei"
        );
        assert_eq!(
            normalizar_origem("https://sei.sistemas.ro.gov.br/sei").unwrap(),
            "https://sei.sistemas.ro.gov.br/sei"
        );
        assert_eq!(
            normalizar_origem("https://sei.sistemas.ro.gov.br/sei/controlador.php?acao=x").unwrap(),
            "https://sei.sistemas.ro.gov.br/sei"
        );
        assert_eq!(
            normalizar_origem(
                "https://sei.sistemas.ro.gov.br/sip/login.php?sigla_orgao_sistema=RO"
            )
            .unwrap(),
            "https://sei.sistemas.ro.gov.br/sei"
        );
    }

    #[test]
    fn extrai_orgaos_via_tl_corretamente() {
        let html = r#"
        <html>
            <body>
                <select name="selOrgao" id="selOrgao">
                    <option value="null">Selecione o órgão</option>
                    <option value="1">AGERO &amp; Cia</option>
                    <option value="9">PGE</option>
                </select>
            </body>
        </html>
        "#;
        let orgaos = extrair_orgaos_do_html(html);
        assert_eq!(orgaos.len(), 2);
        assert_eq!(orgaos[0], ("1".to_string(), "AGERO & Cia".to_string()));
        assert_eq!(orgaos[1], ("9".to_string(), "PGE".to_string()));
    }

    #[test]
    fn extrai_orgaos_ignora_outros_selects() {
        let html = r#"
        <html>
            <body>
                <select name="outraCoisa">
                    <option value="1">Ignorado</option>
                </select>
            </body>
        </html>
        "#;
        let orgaos = extrair_orgaos_do_html(html);
        assert!(orgaos.is_empty());
    }
}
