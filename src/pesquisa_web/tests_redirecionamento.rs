//! RED/GREEN do 302 da pesquisa: `Location` sintético com a forma observada
//! (relativo, `acao=procedimento_trabalhar`), hashes e IDs fictícios.
use super::*;
use ErroRedirecionamentoDaPesquisa as E;

const HASH_ENVIO: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_DESTINO: &str = "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

fn url_do_envio() -> String {
    format!(
        "https://sei.exemplo.gov.br/sei/controlador.php?acao=protocolo_pesquisar&acao_origem=protocolo_pesquisar&infra_sistema=1&infra_unidade_atual=5&infra_hash={HASH_ENVIO}"
    )
}

fn consulta(id: &str) -> String {
    format!(
        "acao=procedimento_trabalhar&acao_origem=protocolo_pesquisar&id_procedimento={id}&id_documento=&infra_sistema=1&infra_unidade_atual=5&infra_hash={HASH_DESTINO}"
    )
}

fn observado() -> String {
    format!("controlador.php?{}", consulta("7"))
}

fn interpretar(localizacao: Option<&str>) -> Result<IdProcedimento, E> {
    interpretar_com_envio(localizacao, &url_do_envio())
}

fn interpretar_com_envio(localizacao: Option<&str>, envio: &str) -> Result<IdProcedimento, E> {
    interpretar_redirecionamento(
        Redirecionamento302::sintetico(localizacao, envio),
        &instalacao(),
    )
    .map(|p| p.id)
}

#[test]
fn url_de_trabalho_sai_resolvida_e_redigida_no_debug() {
    let p = interpretar_redirecionamento(
        Redirecionamento302::sintetico(Some(&observado()), &url_do_envio()),
        &instalacao(),
    )
    .unwrap();
    assert_eq!(
        p.trabalho.as_str(),
        format!(
            "https://sei.exemplo.gov.br/sei/controlador.php?{}",
            consulta("7")
        )
    );
    assert!(!format!("{p:?}").contains(HASH_DESTINO));
}

fn erro(localizacao: &str) -> E {
    interpretar(Some(localizacao)).unwrap_err()
}

#[test]
fn location_relativo_observado_resolve_contra_a_url_do_post_e_devolve_o_id() {
    assert_eq!(interpretar(Some(&observado())).unwrap().as_str(), "7");
}

#[test]
fn location_absoluto_ou_de_raiz_na_mesma_instalacao_e_aceito() {
    for prefixo in [
        "https://sei.exemplo.gov.br/sei/controlador.php?",
        "https://SEI.exemplo.gov.br/sei/controlador.php?",
        "//sei.exemplo.gov.br/sei/controlador.php?",
        "/sei/controlador.php?",
    ] {
        let localizacao = format!("{prefixo}{}", consulta("42"));
        assert_eq!(
            interpretar(Some(&localizacao)).unwrap().as_str(),
            "42",
            "{prefixo}"
        );
    }
}

// Caso de todos tests/test_resolve_http_mesmo_host_010.py @323eb01
#[test]
fn location_http_so_aceita_o_mesmo_host_sem_porta_alternativa() {
    for (origem, esperado) in [
        ("http://SEI.exemplo.gov.br", Ok(())),
        ("http://SEI.exemplo.gov.br:80", Ok(())),
        ("https://sei.exemplo.gov.br", Ok(())),
        (
            "http://outro.gov.br",
            Err(E::LocalizacaoDoRedirecionamentoForaDaInstalacao),
        ),
        (
            "//outro.gov.br",
            Err(E::LocalizacaoDoRedirecionamentoForaDaInstalacao),
        ),
        (
            "http://sei.exemplo.gov.br:8080",
            Err(E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps),
        ),
    ] {
        let localizacao = format!("{origem}/sei/{}", observado());
        assert_eq!(interpretar(Some(&localizacao)).map(|_| ()), esperado);
    }
}

#[test]
fn location_ausente_ou_vazio_e_recusado() {
    assert_eq!(
        interpretar(None).unwrap_err(),
        E::LocalizacaoDoRedirecionamentoAusente
    );
    assert_eq!(erro(""), E::LocalizacaoDoRedirecionamentoAusente);
}

#[test]
fn location_acima_do_limite_e_recusado_antes_de_interpretar() {
    let longo = format!("{}&x={}", observado(), "a".repeat(8_192));
    assert_eq!(erro(&longo), E::LocalizacaoDoRedirecionamentoAcimaDoLimite);
}

#[test]
fn location_para_outro_host_ou_outro_caminho_e_recusado() {
    for prefixo in [
        "https://outro.gov.br/sei/controlador.php?",
        "//outro.gov.br/sei/controlador.php?",
        "https://sei.exemplo.gov.br.outro.gov.br/sei/controlador.php?",
        "/sip/controlador.php?",
        "/sei/modulos/controlador.php?",
        "../controlador.php?",
        "/controlador.php?",
        "outra/controlador.php?",
        "login.php?",
    ] {
        let localizacao = format!("{prefixo}{}", consulta("7"));
        assert_eq!(
            erro(&localizacao),
            E::LocalizacaoDoRedirecionamentoForaDaInstalacao,
            "{prefixo}"
        );
    }
}

#[test]
fn location_sem_https_com_fragmento_ou_caracteres_de_controle_e_recusado() {
    for localizacao in [
        format!("javascript:controlador.php?{}", consulta("7")),
        format!("{}#ancora", observado()),
        format!("{} ", observado()),
        format!("controlador.php?{}\\", consulta("7")),
        "https://".to_owned(),
    ] {
        assert_eq!(
            erro(&localizacao),
            E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps,
            "{localizacao}"
        );
    }
}

#[test]
fn nomes_de_parametro_nao_canonicos_ou_ponto_e_virgula_sao_recusados() {
    for alterado in [
        observado().replace("id_documento=", "id%5Fdocumento="),
        observado().replace("id_documento=", "ID_documento="),
        observado().replace("id_documento=", "id_documento"),
        observado().replace("id_documento=", "id_documento=1;acao=x"),
        format!("{}&&", observado()),
    ] {
        assert_eq!(
            erro(&alterado),
            E::ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento,
            "{alterado}"
        );
    }
}

#[test]
fn acao_diferente_ausente_ou_repetida_e_recusada() {
    for alterado in [
        observado().replace(
            "acao=procedimento_trabalhar",
            "acao=procedimento_visualizar",
        ),
        observado().replace("acao=procedimento_trabalhar&", ""),
        format!("{}&acao=procedimento_trabalhar", observado()),
        format!("{}&acao=procedimento_excluir", observado()),
    ] {
        assert_eq!(
            erro(&alterado),
            E::RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar,
            "{alterado}"
        );
    }
}

#[test]
fn id_procedimento_ausente_vazio_repetido_ou_nao_decimal_e_recusado() {
    let sem_id = observado().replace("id_procedimento=7&", "");
    assert_eq!(erro(&sem_id), E::IdProcedimentoAusenteNoRedirecionamento);
    let vazio = format!("controlador.php?{}", consulta(""));
    assert_eq!(erro(&vazio), E::IdProcedimentoAusenteNoRedirecionamento);
    let repetido = format!("{}&id_procedimento=7", observado());
    assert_eq!(erro(&repetido), E::IdProcedimentoRepetidoNoRedirecionamento);
    for id in ["7a", "-7", "+7", "%37", "0x7"] {
        let alterado = format!("controlador.php?{}", consulta(id));
        assert_eq!(
            erro(&alterado),
            E::IdProcedimentoNaoDecimalNoRedirecionamento,
            "{id}"
        );
    }
}

#[test]
fn infra_hash_ausente_repetido_ou_fora_do_formato_e_recusado() {
    let sem_hash = observado().replace(&format!("&infra_hash={HASH_DESTINO}"), "");
    assert_eq!(erro(&sem_hash), E::InfraHashAusenteNoRedirecionamento);
    let repetido = format!("{}&infra_hash={HASH_DESTINO}", observado());
    assert_eq!(erro(&repetido), E::InfraHashRepetidoNoRedirecionamento);
    for hash in [
        "e".repeat(63),
        "e".repeat(65),
        "g".repeat(64),
        String::new(),
    ] {
        let alterado = observado().replace(HASH_DESTINO, &hash);
        assert_eq!(
            erro(&alterado),
            E::InfraHashForaDoFormatoNoRedirecionamento,
            "{}",
            hash.len()
        );
    }
}

#[test]
fn unidade_atual_do_redirecionamento_precisa_ser_a_do_envio() {
    let sem = observado().replace("&infra_unidade_atual=5", "");
    assert_eq!(erro(&sem), E::UnidadeAtualAusenteNoRedirecionamento);
    let repetida = format!("{}&infra_unidade_atual=5", observado());
    assert_eq!(erro(&repetida), E::UnidadeAtualRepetidaNoRedirecionamento);
    let outra = observado().replace("infra_unidade_atual=5", "infra_unidade_atual=6");
    assert_eq!(
        erro(&outra),
        E::UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio
    );
    let envio_sem = url_do_envio().replace("&infra_unidade_atual=5", "");
    assert_eq!(
        interpretar_com_envio(Some(&observado()), &envio_sem).unwrap_err(),
        E::UnidadeAtualAusenteNaUrlDoEnvio
    );
    let envio_repetido = format!("{}&infra_unidade_atual=5", url_do_envio());
    assert_eq!(
        interpretar_com_envio(Some(&observado()), &envio_repetido).unwrap_err(),
        E::UnidadeAtualRepetidaNaUrlDoEnvio
    );
}

#[test]
fn acao_de_origem_ausente_repetida_ou_diferente_da_pesquisa_e_recusada() {
    let sem = observado().replace("&acao_origem=protocolo_pesquisar", "");
    assert_eq!(erro(&sem), E::AcaoDeOrigemAusenteNoRedirecionamento);
    let repetida = format!("{}&acao_origem=protocolo_pesquisar", observado());
    assert_eq!(erro(&repetida), E::AcaoDeOrigemRepetidaNoRedirecionamento);
    for origem in ["procedimento_controlar", "", "PROTOCOLO_PESQUISAR"] {
        let outra = observado().replace(
            "acao_origem=protocolo_pesquisar",
            &format!("acao_origem={origem}"),
        );
        assert_eq!(
            erro(&outra),
            E::AcaoDeOrigemDoRedirecionamentoDiferenteDaPesquisa,
            "{origem}"
        );
    }
}

#[test]
fn infra_sistema_do_redirecionamento_precisa_ser_decimal_e_o_do_envio() {
    let sem = observado().replace("&infra_sistema=1", "");
    assert_eq!(erro(&sem), E::InfraSistemaAusenteNoRedirecionamento);
    let repetido = format!("{}&infra_sistema=1", observado());
    assert_eq!(erro(&repetido), E::InfraSistemaRepetidoNoRedirecionamento);
    for valor in ["", "1a", "-1", "%31", "+1"] {
        let alterado = observado().replace("infra_sistema=1", &format!("infra_sistema={valor}"));
        assert_eq!(
            erro(&alterado),
            E::InfraSistemaNaoDecimalNoRedirecionamento,
            "{valor}"
        );
    }
    // Comparação exata: `01` não é normalizado para `1`.
    for valor in ["2", "01"] {
        let outro = observado().replace("infra_sistema=1", &format!("infra_sistema={valor}"));
        assert_eq!(
            erro(&outro),
            E::InfraSistemaDoRedirecionamentoDiferenteDoEnvio,
            "{valor}"
        );
    }
}

#[test]
fn infra_sistema_ausente_repetido_ou_nao_decimal_na_url_do_envio_falha_fechado() {
    let casos = [
        (
            url_do_envio().replace("&infra_sistema=1", ""),
            E::InfraSistemaAusenteNaUrlDoEnvio,
        ),
        (
            format!("{}&infra_sistema=1", url_do_envio()),
            E::InfraSistemaRepetidoNaUrlDoEnvio,
        ),
        (
            url_do_envio().replace("infra_sistema=1", "infra_sistema=x"),
            E::InfraSistemaNaoDecimalNaUrlDoEnvio,
        ),
    ];
    for (envio, esperado) in casos {
        assert_eq!(
            interpretar_com_envio(Some(&observado()), &envio).unwrap_err(),
            esperado,
            "{envio}"
        );
    }
}

#[test]
fn subcodigos_sao_estaveis_e_distintos() {
    let todos = [
        E::LocalizacaoDoRedirecionamentoAusente,
        E::LocalizacaoDoRedirecionamentoAcimaDoLimite,
        E::LocalizacaoDoRedirecionamentoNaoEhUrlHttps,
        E::LocalizacaoDoRedirecionamentoForaDaInstalacao,
        E::ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento,
        E::RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar,
        E::IdProcedimentoAusenteNoRedirecionamento,
        E::IdProcedimentoRepetidoNoRedirecionamento,
        E::IdProcedimentoNaoDecimalNoRedirecionamento,
        E::InfraHashAusenteNoRedirecionamento,
        E::InfraHashRepetidoNoRedirecionamento,
        E::InfraHashForaDoFormatoNoRedirecionamento,
        E::UnidadeAtualAusenteNoRedirecionamento,
        E::UnidadeAtualRepetidaNoRedirecionamento,
        E::UnidadeAtualAusenteNaUrlDoEnvio,
        E::UnidadeAtualRepetidaNaUrlDoEnvio,
        E::UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio,
        E::AcaoDeOrigemAusenteNoRedirecionamento,
        E::AcaoDeOrigemRepetidaNoRedirecionamento,
        E::AcaoDeOrigemDoRedirecionamentoDiferenteDaPesquisa,
        E::InfraSistemaAusenteNoRedirecionamento,
        E::InfraSistemaRepetidoNoRedirecionamento,
        E::InfraSistemaNaoDecimalNoRedirecionamento,
        E::InfraSistemaAusenteNaUrlDoEnvio,
        E::InfraSistemaRepetidoNaUrlDoEnvio,
        E::InfraSistemaNaoDecimalNaUrlDoEnvio,
        E::InfraSistemaDoRedirecionamentoDiferenteDoEnvio,
    ];
    let unicos: std::collections::HashSet<_> = todos
        .iter()
        .map(|e| crate::erro_publico::rotulo(&e))
        .collect();
    assert_eq!(unicos.len(), todos.len());
}

#[test]
fn redirect_de_documento_na_pesquisa_rapida_nao_aceita_id_protocolo() {
    let envio = url_do_envio().replace("protocolo_pesquisar", "protocolo_pesquisa_rapida");
    let destino = format!(
        "controlador.php?acao=procedimento_trabalhar&acao_origem=protocolo_pesquisa_rapida&id_documento=42&infra_sistema=1&infra_unidade_atual=5&infra_hash={HASH_DESTINO}"
    );
    let conferir = |alvo: &str| {
        conferir_trabalho(
            Some(alvo),
            &envio,
            &instalacao(),
            OrigemDoTrabalho::PesquisaRapidaDocumento,
        )
        .map(|redirecionado| redirecionado.id)
    };
    assert_eq!(conferir(&destino).unwrap().as_str(), "42");
    assert_eq!(
        conferir(&destino.replace("id_documento=42", "id_protocolo=42")),
        Err(E::IdDocumentoAusenteNoRedirecionamento)
    );
    assert_eq!(
        conferir(&format!("{destino}&id_documento=42")),
        Err(E::IdDocumentoRepetidoNoRedirecionamento)
    );
    assert_eq!(
        conferir(&destino.replace("id_documento=42", "id_documento=x")),
        Err(E::IdDocumentoNaoDecimalNoRedirecionamento)
    );
}
