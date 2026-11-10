//! RED/GREEN da sessão depois da troca (#205): a URL do Controle novo vira
//! a da sessão, o marcador segue a mesma geração com a URL nova, e nenhum
//! link da unidade anterior volta do índice; o Controle novo é colhido com
//! a chave da unidade nova. Arquivos e hashes sintéticos.
use super::*;
use crate::indice_local::{ChaveDoLink, NOME_DO_ARQUIVO, unidade_dos_links};
use crate::sessao_login::geracao_da_sessao;
use crate::unidade_trocar::tests_apoio::{
    COOKIE, ID_A, ID_B, ORIGEM, controle, url_controle, url_limpa,
};

fn privado(caminho: &std::path::Path, conteudo: &str) {
    std::fs::write(caminho, conteudo).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
}

fn pasta() -> std::path::PathBuf {
    let pasta = std::env::temp_dir().join(format!(
        "ies-trocar-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&pasta).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&pasta, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
    pasta
}

#[test]
fn falhas_da_aplicacao_preservam_a_causa_tipificada() {
    let dir = pasta();
    let arquivos = ArquivosDeAcesso {
        url_controle: dir.join("url"),
        cookie: dir.join("cookie"),
        origem: dir.join("origem"),
    };
    let indice = IndiceLocal::em(dir.join(NOME_DO_ARQUIVO));
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let novo = ControleLido {
        url: url_limpa(ID_B),
        html: controle("PR-B", ID_B),
        mime: "text/html; charset=iso-8859-1".into(),
    };

    let cookie_ausente = CookieDaSessao::ler(&dir.join("cookie-inexistente"));
    assert!(matches!(
        aplicar_troca(&arquivos, &indice, &instalacao, &cookie_ausente, &novo),
        Err(FalhaAplicarTroca::Cookie(
            FalhaTransporte::ArquivoPrivadoAusenteOuInacessivel
        ))
    ));

    std::fs::create_dir(&arquivos.url_controle).unwrap();
    assert!(matches!(
        aplicar_troca(
            &arquivos,
            &indice,
            &instalacao,
            &CookieDaSessao::sintetico(COOKIE),
            &novo
        ),
        Err(FalhaAplicarTroca::Gravacao(_))
    ));
    std::fs::remove_dir_all(dir).unwrap();
}

fn link_de_trabalho(id_unidade: &str) -> String {
    format!(
        "{ORIGEM}/controlador.php?acao=procedimento_trabalhar&id_procedimento=7&infra_sistema=100000100&infra_unidade_atual={id_unidade}&infra_hash={}",
        "d".repeat(64)
    )
}

#[test]
fn troca_regrava_sessao_e_nada_da_unidade_anterior_e_reaproveitado() {
    let dir = pasta();
    let arquivos = ArquivosDeAcesso {
        url_controle: dir.join("url"),
        cookie: dir.join("cookie"),
        origem: dir.join("origem"),
    };
    let antiga = url_controle(ID_A);
    privado(&arquivos.url_controle, &format!("{antiga}\n"));
    privado(&arquivos.cookie, COOKIE);
    privado(&arquivos.origem, ORIGEM);
    let geracao = geracao_da_sessao(COOKIE);
    let marcador = json!({"geracao": 3, "instalacao": ORIGEM, "cookie_sha256": geracao,
                          "url_controle_sha256": geracao_da_sessao(&antiga)});
    privado(&dir.join("url.geracao"), &format!("{marcador}\n"));
    let indice = IndiceLocal::em(dir.join(NOME_DO_ARQUIVO));
    let instalacao = Instalacao::interpretar(ORIGEM).unwrap();
    let chave = |unidade| ChaveDoLink {
        geracao_sessao: &geracao,
        unidade,
        acao: "procedimento_trabalhar",
        objeto: "7",
    };
    indice
        .registrar_link(&instalacao, chave(ID_A), &link_de_trabalho(ID_A))
        .unwrap();
    // Um link de B guardado numa visita anterior a B também não volta.
    indice
        .registrar_link(&instalacao, chave(ID_B), &link_de_trabalho(ID_B))
        .unwrap();
    assert!(indice.link(&instalacao, chave(ID_A)).is_some());
    let novo = ControleLido {
        url: url_limpa(ID_B),
        html: controle("PR-B", ID_B),
        mime: "text/html; charset=iso-8859-1".into(),
    };
    assert_eq!(
        aplicar_troca(
            &arquivos,
            &indice,
            &instalacao,
            &CookieDaSessao::sintetico(COOKIE),
            &novo,
        ),
        Ok(Some(2))
    );
    let gravada = std::fs::read_to_string(&arquivos.url_controle).unwrap();
    assert_eq!(gravada.trim(), url_limpa(ID_B));
    assert_eq!(unidade_dos_links(gravada.trim()).as_deref(), Some(ID_B));
    assert!(crate::sessao_login::mesma_geracao(
        &arquivos,
        &url_limpa(ID_B),
        COOKIE
    ));
    assert!(!crate::sessao_login::mesma_geracao(
        &arquivos, &antiga, COOKIE
    ));
    let lido: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("url.geracao")).unwrap()).unwrap();
    assert_eq!(lido["geracao"], 3);
    assert_eq!(std::fs::read_to_string(&arquivos.cookie).unwrap(), COOKIE);
    assert_eq!(indice.link(&instalacao, chave(ID_A)), None);
    assert_eq!(indice.link(&instalacao, chave(ID_B)), None);
    // A URL de 192 da sessão nova é lida pelos outros comandos.
    let referencias =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie);
    assert!(referencias.is_ok());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn sucesso_publica_escopo_da_sessao_e_so_nomes_dos_campos() {
    let unidade = |s: &str, id: &str| UnidadeOferecida {
        sigla: s.into(),
        descricao: format!("Unidade {s}"),
        id_unidade: id.into(),
    };
    let lido = |url: String| ControleLido {
        url,
        html: String::new(),
        mime: String::new(),
    };
    let mut troca = Troca {
        antes: "PR-A".into(),
        pedida: "PR-B".into(),
        escolhida: Some(unidade("PR-B", ID_B)),
        oferecidas: vec![unidade("PR-A", ID_A), unidade("PR-B", ID_B)],
        campos_do_envio: Some(vec!["hdnInfraTipoPagina".into(), "selInfraUnidades".into()]),
        confirmado: true,
        controle_inicial: lido(url_controle(ID_A)),
        controle_novo: Some(lido(url_limpa(ID_B))),
    };
    let valor = json_de_sucesso(&troca, Some(4), 5);
    assert_eq!(valor["depois"], "PR-B");
    assert_eq!(valor["alterado"], true);
    assert_eq!(valor["escopo"], "sessao_da_conta");
    assert_eq!(valor["links_descartados"], 4);
    assert_eq!(valor["requisicoes"], 5);
    assert_eq!(valor["envio"]["campos"][1], "selInfraUnidades");
    let mut bytes = Vec::new();
    serde_json::to_writer_pretty(&mut bytes, &resultado_troca(&troca, Some(4), 5)).unwrap();
    assert_eq!(
        std::str::from_utf8(&bytes).unwrap(),
        serde_json::to_string_pretty(&valor).unwrap()
    );
    let texto = valor.to_string();
    assert!(!texto.contains("infra_hash") && !texto.contains("https://"));
    assert!(humano(&troca, 5).contains("toda a sessão da conta"));

    // Já na unidade: oferecidas e envio devem continuar null, não [].
    troca.escolhida = None;
    troca.campos_do_envio = None;
    troca.controle_novo = None;
    let legado = json_de_sucesso(&troca, None, 2);
    let direto = serde_json::to_value(resultado_troca(&troca, None, 2)).unwrap();
    assert_eq!(direto, legado);
    assert!(direto["oferecidas"].is_null());
    assert!(direto["envio"].is_null());
    assert_eq!(direto["depois"], "PR-B");
}

#[test]
fn falhas_de_troca_sao_apresentadas_direto_do_tipo_sem_json() {
    let u = |sigla: &str| UnidadeOferecida {
        sigla: sigla.to_owned(),
        descricao: format!("Unidade {sigla}"),
        id_unidade: "7".into(),
    };
    let nao_oferecida = ErroTroca::UnidadeNaoOferecida {
        oferecidas: vec![u("PR-A"), u("PR-B")],
    };
    let linhas = linhas_da_falha(&nao_oferecida, 2);
    assert_eq!(linhas[1], "IES: oferecidas: PR-A, PR-B");
    assert_eq!(linhas[2], "IES: 2 requisições.");

    let ambigua = ErroTroca::SiglaDeUnidadeAmbigua {
        candidatas: vec![u("PR-A"), u("PR-A")],
    };
    let linhas = linhas_da_falha(&ambigua, 1);
    assert_eq!(linhas[1], "IES: candidatas: PR-A, PR-A");
    assert_eq!(linhas[2], "IES: 1 requisições.");

    let incerta = ErroTroca::NaoConfirmado {
        etapa: crate::unidade_trocar::EtapaDaTroca::SegundoRedirecionamento,
        falha: crate::unidade_trocar::FalhaDepoisDoEnvio::RedirecionamentoSemLocalizacao,
    };
    let linhas = linhas_da_falha(&incerta, 3);
    assert_eq!(linhas[2], "IES: 3 requisições.");
}
