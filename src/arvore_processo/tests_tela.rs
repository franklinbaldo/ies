//! RED/GREEN da tela `procedimento_trabalhar`: unidade e `iframe#ifrArvore`
//! na forma observada (`UnB` e GO, `&` cru no `src`). HTML sintético.
use super::tela::TelaDoProcesso;
use super::*;
use crate::instalacao::Instalacao;
use std::fmt::Write as _;

pub(crate) const HASH_ARVORE: &str =
    "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";

pub(crate) fn instalacao() -> Instalacao {
    Instalacao::interpretar("https://sei.exemplo.gov.br/sei").unwrap()
}

pub(crate) fn src_da_arvore(id: &str) -> String {
    format!(
        "controlador.php?acao=procedimento_visualizar&acao_origem=procedimento_trabalhar&acao_retorno=principal&id_procedimento={id}&id_documento=&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH_ARVORE}"
    )
}

pub(crate) fn tela_do_processo(unidade: &str, iframes: &[String]) -> String {
    let mut html = format!(
        "<html><head><title>SEI - Processo</title></head><body>\
         <a id=\"lnkInfraUnidade\" href=\"#\">{unidade}</a>"
    );
    for src in iframes {
        let _ = write!(
            html,
            "<iframe id=\"ifrArvore\" name=\"ifrArvore\" class=\"c\" src=\"{src}\"></iframe>"
        );
    }
    html.push_str("<iframe id=\"ifrVisualizacao\" src=\"about:blank\"></iframe></body></html>");
    html
}

fn ler(html: &str, unidade: Option<&str>) -> Result<TelaDoProcesso, super::ErroTelaDoProcesso> {
    ler_tela_do_processo(&instalacao(), html, "10", unidade)
}

#[test]
fn unidade_e_endereco_absoluto_da_arvore() {
    let html = tela_do_processo("UNIDADE-TESTE", &[src_da_arvore("10")]);
    let tela = ler(&html, None).unwrap();
    assert_eq!(tela.unidade, "UNIDADE-TESTE");
    assert_eq!(
        tela.arvore.as_str(),
        format!("https://sei.exemplo.gov.br/sei/{}", src_da_arvore("10"))
    );
    assert!(!format!("{tela:?}").contains(HASH_ARVORE));
    assert!(ler(&html, Some(" UNIDADE-TESTE ")).is_ok());
    assert_eq!(
        ler(&html, Some("OUTRA")),
        Err(super::ErroTelaDoProcesso::UnidadeDaTelaDiferenteDaSolicitada)
    );
}

#[test]
fn src_com_entidade_amp_e_aceito() {
    let html = tela_do_processo("U", &[src_da_arvore("10").replace('&', "&amp;")]);
    assert!(ler(&html, None).is_ok());
}

#[test]
fn recusas_da_tela() {
    let sem_unidade = tela_do_processo("U", &[src_da_arvore("10")])
        .replace("<a id=\"lnkInfraUnidade\" href=\"#\">U</a>", "");
    assert_eq!(
        ler(&sem_unidade, None),
        Err(super::ErroTelaDoProcesso::IdentificacaoDaUnidadeAusenteNaTelaDoProcesso)
    );
    assert_eq!(
        ler(&tela_do_processo("U", &[]), None),
        Err(super::ErroTelaDoProcesso::ArvoreAusenteNaTelaDoProcesso)
    );
    assert_eq!(
        ler(
            &tela_do_processo("U", &[src_da_arvore("10"), src_da_arvore("10")]),
            None
        ),
        Err(super::ErroTelaDoProcesso::ArvoresRepetidasNaTelaDoProcesso)
    );
    assert_eq!(
        ler(&tela_do_processo("U", &[src_da_arvore("11")]), None),
        Err(super::ErroTelaDoProcesso::EnderecoDaArvoreDeOutroProcesso)
    );
    for src in [
        src_da_arvore("10").replace("procedimento_visualizar", "arvore_visualizar"),
        src_da_arvore("10").replace(HASH_ARVORE, "curto"),
        format!("{}&acao=procedimento_visualizar", src_da_arvore("10")),
        format!("https://outro.exemplo.gov.br/sei/{}", src_da_arvore("10")),
        src_da_arvore("10").replace("id_procedimento=10&", ""),
        "javascript:void(0)".to_owned(),
    ] {
        assert_eq!(
            ler(&tela_do_processo("U", std::slice::from_ref(&src)), None),
            Err(super::ErroTelaDoProcesso::EnderecoDaArvoreNaoReconhecido),
            "{src}"
        );
    }
}

#[test]
fn arvore_vazia_com_validacao_de_acesso_e_erro_proprio() {
    // GO 5.0.3 (live de 10/11/2026, processo 202600005006024): o SEI abre a
    // modal `usuario_validar_acesso` (pede a senha) e serve a árvore vazia.
    let html = tela_do_processo("SEAD/CDE-09516", &[String::new()]).replace(
        "</body>",
        "<script>function inicializar(){ if (''!='1'){ infraAbrirJanelaModal('controlador.php?acao=usuario_validar_acesso&acao_origem=procedimento_trabalhar&id_procedimento=10&infra_hash=SINTETICO',500,300,true,'finalizar'); return; } }</script></body>",
    );
    assert_eq!(
        ler(&html, None),
        Err(super::ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso)
    );
    // Sem a modal, árvore vazia continua sendo endereço não reconhecido.
    let sem_modal = tela_do_processo("SEAD/CDE-09516", &[String::new()]);
    assert!(matches!(
        ler(&sem_modal, None),
        Err(super::ErroTelaDoProcesso::EnderecoDaArvoreNaoReconhecido)
    ));
}
