use super::*;
use std::io::Cursor;

fn url(host: &str) -> String {
    format!(
        "https://{host}/sei/controlador.php?acao=procedimento_controlar&infra_hash={}",
        "a".repeat(64)
    )
}

#[test]
fn http_200_com_formulario_de_login_nao_eh_caixa_autenticada() {
    let corpo = b"<form id='frmLogin'><input name='txtUsuario'></form>";
    let resposta = RespostaWeb {
        status_code: 200,
        mime: "text/html; charset=utf-8".to_owned(),
        content_length: None,
        corpo: Cursor::new(corpo),
    };
    assert_eq!(
        decodificar_resposta(resposta),
        Err(FalhaTransporte::PaginaDeLoginRecebidaNoControle)
    );
}

#[test]
fn nao_aceita_url_sem_assinatura() {
    assert!(validar_url("http://sei.exemplo/controlador.php?acao=procedimento_controlar").is_err());
    assert!(
        validar_url("https://sei.exemplo/controlador.php?acao=procedimento_controlar").is_err()
    );
    assert!(validar_url(&url("usuario@sei.exemplo")).is_err());
    assert!(validar_url(&url("sei.exemplo:443")).is_err());
    assert!(validar_url(&format!("{}&param=\r\nCookie:unsafe", url("sei.exemplo"))).is_err());
    assert!(validar_url(&url("sei.exemplo\\outro")).is_err());
}

#[test]
fn permite_somente_url_assinada_da_caixa() {
    let url = url("sei.exemplo");
    assert!(validar_url(&url).is_ok());
    assert!(validar_url(&url.replace("procedimento_controlar", "procedimento_excluir")).is_err());
    assert!(validar_url(&format!("{url}&acao=procedimento_excluir")).is_err());
    assert!(validar_url(&format!("{url}&infra_hash={}", "b".repeat(64))).is_err());
    assert!(validar_url(&format!("{url}&a%63ao=procedimento_excluir")).is_err());
}

#[test]
fn host_de_cookies_precisa_conferir_antes_da_rede() {
    let legitimo = url("sei.treinamento.unb.br");
    let adulterado = url("sei.exemplo");
    assert_eq!(
        validar_origem(&legitimo, "https://sei.treinamento.unb.br/sei"),
        Ok(())
    );
    assert_eq!(
        validar_origem(&legitimo, "https://SEI.TREINAMENTO.UNB.BR/sei/"),
        Ok(())
    );
    assert_eq!(
        validar_origem(&adulterado, "https://sei.treinamento.unb.br/sei"),
        Err(FalhaTransporte::OrigemNaoAutorizada)
    );
    assert_eq!(
        validar_origem(&legitimo, "https://sei.treinamento.unb.br.evil.tld/sei"),
        Err(FalhaTransporte::OrigemNaoAutorizada)
    );
    assert_eq!(
        validar_origem(&legitimo, "http://sei.treinamento.unb.br/sei"),
        Err(FalhaTransporte::OrigemNaoAutorizada)
    );
}

#[test]
fn corpo_e_limitado_durante_a_leitura() {
    assert_eq!(
        ler_limitado(Cursor::new(vec![42; 10]), 10).unwrap().len(),
        10
    );
    assert_eq!(
        ler_limitado(Cursor::new(vec![42; 11]), 10),
        Err(FalhaTransporte::RespostaExcessiva)
    );
    assert_eq!(
        ler_limitado(Cursor::new(vec![42; 10_000]), 10),
        Err(FalhaTransporte::RespostaExcessiva)
    );
}

#[cfg(unix)]
#[test]
fn links_simbolicos_para_arquivos_privados_nao_podem_ser_lidos() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let raiz = std::env::temp_dir().join(format!(
        "ies-privados-symlink-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&raiz).unwrap();
    let original = raiz.join("cookie-original");
    fs::write(&original, "PHPSESSID=DEMONSTRACAO").unwrap();
    fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
    let atalho = raiz.join("cookie-atalho");
    symlink(&original, &atalho).unwrap();
    assert_eq!(
        arquivo_privado(&atalho, 8_192),
        Err(FalhaTransporte::ArquivoPrivadoLinkSimbolico)
    );
    assert_eq!(
        arquivo_privado(&original, 8_192),
        Ok("PHPSESSID=DEMONSTRACAO".to_owned())
    );
    fs::remove_dir_all(raiz).unwrap();
}

#[cfg(unix)]
#[test]
fn url_assinada_via_link_simbolico_falha_antes_de_consultar_rede() {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let raiz = std::env::temp_dir().join(format!(
        "ies-url-symlink-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&raiz).unwrap();
    let original = raiz.join("url-original");
    fs::write(&original, url("sei.exemplo")).unwrap();
    fs::set_permissions(&original, fs::Permissions::from_mode(0o600)).unwrap();
    let atalho = raiz.join("url-atalho");
    symlink(&original, &atalho).unwrap();
    assert_eq!(
        read_box_from_file(
            &atalho,
            &raiz.join("cookie-nao-existe"),
            &raiz.join("origem-nao-existe"),
        ),
        Err(FalhaTransporte::ArquivoPrivadoLinkSimbolico)
    );
    fs::remove_dir_all(raiz).unwrap();
}

#[cfg(unix)]
#[test]
fn descritor_revalida_permissoes_e_limite_de_conteudo() {
    use std::os::unix::fs::PermissionsExt;
    let pasta = std::env::temp_dir().join(format!(
        "ies-descritor-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&pasta).unwrap();
    let arquivo = pasta.join("privado");
    fs::write(&arquivo, "CONTEUDO").unwrap();
    fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        arquivo_privado(&arquivo, 128),
        Err(FalhaTransporte::ArquivoPrivadoComPermissoesInseguras)
    );
    fs::set_permissions(&arquivo, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(
        arquivo_privado(&arquivo, 4),
        Err(FalhaTransporte::ArquivoPrivadoAcimaDoLimite)
    );
    assert_eq!(arquivo_privado(&arquivo, 128), Ok("CONTEUDO".to_owned()));
    fs::remove_dir_all(&pasta).unwrap();
}

#[test]
fn mesma_origem_em_outra_pasta_nao_recebe_cookie() {
    let legitimo = url("sei.exemplo");
    assert_eq!(
        validar_origem(&legitimo, "https://sei.exemplo"),
        Err(FalhaTransporte::OrigemNaoAutorizada)
    );
    assert_eq!(
        validar_origem(&legitimo, "https://sei.exemplo/outro"),
        Err(FalhaTransporte::OrigemNaoAutorizada)
    );
    assert_eq!(
        validar_origem(&legitimo, "https://sei.exemplo/sei\nversao=5.0.4"),
        Ok(())
    );
    assert_eq!(
        validar_origem(&legitimo, "https://sei.exemplo/sei\nversao=4.0.9"),
        Ok(())
    );
}

#[test]
fn nomes_que_php_normaliza_nao_podem_sobrescrever_acao_ou_hash() {
    let legitima = url("sei.exemplo");
    for alias in [
        "a%63ao=procedimento_excluir",
        "infra%5Fhash=token_falso",
        "infra.hash=token_falso",
        "infra+hash=token_falso",
        "acao.ajax=excluir",
    ] {
        assert_eq!(
            validar_url(&format!("{legitima}&{alias}")),
            Err(FalhaTransporte::ChaveDeConsultaNaoCanonica),
            "{alias}"
        );
    }
}

#[test]
fn ponto_e_virgula_na_query_nao_muda_acao_apos_parse_php() {
    let legitima = url("sei.exemplo");
    assert_eq!(
        validar_url(&format!(
            "{legitima}&origem=publica;acao=procedimento_excluir"
        )),
        Err(FalhaTransporte::SeparadorDeConsultaAmbiguo)
    );
}

#[test]
fn parametros_auxiliares_canonicos_preservam_get_unico() {
    let legitima = url("sei.exemplo");
    assert!(validar_url(&format!("{legitima}&id_procedimento=42")).is_ok());
}
