//! RED/GREEN da tela "Conclusão de Processo" (lista fechada de controles,
//! opção e processo), do corpo derivado pelo primitivo e do 302 esperado.
//! Telas sintéticas na forma das cinco instalações 4.1/5.0.
use super::tela::{
    CAMPOS_DO_ENVIO, CONCLUIR_PROCESSO, conferir_tela_de_conclusao, redirecionamento_esperado,
};
use super::tests_apoio::*;
use crate::arvore_processo::tests_tela::instalacao;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{CodificacaoDaTela, TelaServida, derivar_submissao};

fn conferir(html: &str) -> Result<(), super::erro::ErroTelaDeConclusao> {
    conferir_tela_de_conclusao(html, ID)
}

#[test]
fn tela_observada_passa_e_o_corpo_e_o_do_navegador() {
    let html = tela_de_conclusao(ID);
    assert_eq!(conferir(&html), Ok(()));
    let url = absoluta(&link_de_concluir(ID));
    let tela = TelaServida {
        url: &url,
        html: &html,
        codificacao: CodificacaoDaTela::Iso88591,
    };
    let submissao = derivar_submissao(
        &CONCLUIR_PROCESSO,
        &instalacao(),
        &tela,
        UNIDADE,
        &CAMPOS_DO_ENVIO,
    )
    .unwrap();
    assert_eq!(submissao.url(), absoluta(&action_de_concluir()));
    assert_eq!(
        String::from_utf8(submissao.corpo().to_vec()).unwrap(),
        CORPO_DA_CONCLUSAO
    );
}

#[test]
fn controle_fora_da_lista_e_recusado() {
    let html = tela_de_conclusao(ID);
    let extra = html.replace(
        "</form>",
        "<input type=\"text\" name=\"txtMotivo\" value=\"\" /></form>",
    );
    assert_eq!(
        conferir(&extra),
        Err(super::erro::ErroTelaDeConclusao::Html(ErroHtml::valor(
            "campo_nao_previsto_na_tela_de_conclusao"
        )))
    );
    let oculto = html.replace(
        "</form>",
        "<input type=\"hidden\" name=\"hdnOutro\" value=\"1\" /></form>",
    );
    assert_eq!(
        conferir(&oculto),
        Err(super::erro::ErroTelaDeConclusao::Html(ErroHtml::valor(
            "campo_nao_previsto_na_tela_de_conclusao"
        )))
    );
    // Botão `button` e controle desabilitado não participam do envio.
    let neutros = html.replace(
        "</form>",
        "<button type=\"button\" name=\"btnX\">x</button><input type=\"text\" name=\"txtY\" disabled=\"disabled\" /></form>",
    );
    assert_eq!(conferir(&neutros), Ok(()));
}

#[test]
fn sem_formulario_de_outro_processo_ou_sem_somente_concluir() {
    assert_eq!(
        conferir("<html><body></body></html>"),
        Err(super::erro::ErroTelaDeConclusao::Html(ErroHtml::ausente(
            "formulario_de_conclusao_ausente"
        )))
    );
    assert_eq!(
        conferir_tela_de_conclusao(&tela_de_conclusao("11"), ID),
        Err(super::erro::ErroTelaDeConclusao::TelaDeConclusaoDeOutroProcesso)
    );
    let html = tela_de_conclusao(ID);
    let desmarcada = html.replace("value=\"S\" checked=\"checked\"", "value=\"S\"");
    assert_eq!(
        conferir(&desmarcada),
        Err(super::erro::ErroTelaDeConclusao::OpcaoSomenteConcluirNaoMarcada)
    );
    let agendar = desmarcada.replace("value=\"V\" ", "value=\"V\" checked=\"checked\" ");
    assert_eq!(
        conferir(&agendar),
        Err(super::erro::ErroTelaDeConclusao::OpcaoSomenteConcluirNaoMarcada)
    );
    let sem_s = html.replace(
        "value=\"S\" checked=\"checked\"",
        "value=\"X\" checked=\"checked\"",
    );
    assert_eq!(
        conferir(&sem_s),
        Err(super::erro::ErroTelaDeConclusao::OpcaoSomenteConcluirNaoMarcada)
    );
}

#[test]
fn reabertura_preenchida_ou_processo_selecionado_e_recusa() {
    let html = tela_de_conclusao(ID);
    for preenchida in [
        html.replace(
            "value=\"1\" class=\"infraRadio\"/>",
            "value=\"1\" class=\"infraRadio\" checked=\"checked\"/>",
        ),
        html.replace(
            "class=\"infraText\" value=\"\" tabindex=\"505\"",
            "class=\"infraText\" value=\"01/01/2027\" tabindex=\"505\"",
        ),
        html.replace(
            "class=\"infraCheckbox\"  tabindex",
            "class=\"infraCheckbox\" checked=\"checked\" tabindex",
        ),
        html.replace(
            &format!("<option value=\"{ID}\">"),
            &format!("<option value=\"{ID}\" selected=\"selected\">"),
        ),
    ] {
        assert_eq!(
            conferir(&preenchida),
            Err(super::erro::ErroTelaDeConclusao::TelaDeConclusaoComReaberturaPreenchida)
        );
    }
}

#[test]
fn redirecionamento_esperado_so_para_a_arvore_ou_o_controle_desta_conclusao() {
    let post = absoluta("controlador.php?acao=procedimento_concluir");
    let esperado = |l: Option<&str>, id: &str| redirecionamento_esperado(l, &post, id);
    let arvore = volta_a_arvore();
    assert!(esperado(Some(&arvore), ID));
    // A forma gravada é a relativa; no host e na pasta do POST, `http://`
    // (medido pelo Todos no GO, issue #766) ou `https://`, é o mesmo
    // controlador. Outro host não é.
    assert!(esperado(Some(&absoluta(&arvore)), ID));
    assert!(esperado(
        Some(&absoluta(&arvore).replacen("https://", "http://", 1)),
        ID
    ));
    assert!(!esperado(
        Some(&absoluta(&arvore).replacen("sei.exemplo.gov.br", "outro.exemplo.gov.br", 1)),
        ID
    ));
    let principal = arvore.replace("acao=arvore_visualizar", "acao=principal");
    assert!(esperado(Some(&principal), ID));
    assert!(!esperado(None, ID));
    assert!(!esperado(Some(&arvore), "11"));
    let outra_origem = arvore.replace("acao_origem=procedimento_concluir", "acao_origem=x");
    assert!(!esperado(Some(&outra_origem), ID));
    let outro_destino = arvore.replace("acao=arvore_visualizar", "acao=procedimento_controlar");
    assert!(!esperado(Some(&outro_destino), ID));
    assert!(!esperado(
        Some("https://sip.exemplo.gov.br/sip/login.php"),
        ID
    ));
}
