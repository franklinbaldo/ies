//! RED/GREEN da coluna Marcadores da tabela D: um `a.ancMarcador` por
//! marcador (estrutura da `UnB` 4.0.9, 04/11/2026), lista sempre, nunca
//! texto concatenado. Âncoras SINTÉTICAS: nomes, textos, cores, ids e
//! hashes inventados.
use super::tests_colunas::{ler, rotulo, tabela};
use super::*;
use crate::erro_html::ErroHtml;
use serde_json::json;

const HASH: &str = "5555555555555555555555555555555555555555555555555555555555555555";

/// Um marcador na forma servida: `href` com `#ID-<id>`, tooltip com o
/// texto como literal JS de aspas simples, ícone e nome em dois `div`.
pub(crate) fn ancora(id_procedimento: &str, id: &str, cor: &str, dica: &str, nome: &str) -> String {
    format!(
        "<a class=\"ancMarcador\" href=\"controlador.php?acao=andamento_marcador_gerenciar&acao_origem=procedimento_controlar&acao_retorno=procedimento_controlar&id_procedimento={id_procedimento}&infra_sistema=100000100&infra_unidade_atual=110000001&infra_hash={HASH}#ID-{id}\" \
         onclick=\"infraLimparFormatarTrAcessada(this.parentNode.parentNode);\" \
         onmouseover=\"return infraTooltipMostrar('{dica}');\" onmouseout=\"return infraTooltipOcultar();\">\
         <div><img src=\"svg/marcador_{cor}.svg?11\" class=\"imagemStatus\"/></div><div>{nome}</div></a>"
    )
}

fn celula(conteudo: &str) -> String {
    format!("<td align=\"left\" valign=\"top\">{conteudo}</td>")
}

fn com_marcadores(conteudo: &str) -> Result<LeituraDetalhada, ErroDetalhada> {
    ler(&tabela(&[rotulo("Marcadores")], &[&celula(conteudo)]))
}

fn marcadores(conteudo: &str) -> serde_json::Value {
    let leitura = com_marcadores(conteudo).unwrap();
    serde_json::to_value(&leitura.processos[0]).unwrap()["colunas"]["marcadores"].clone()
}

#[test]
fn dois_marcadores_na_mesma_celula_saem_como_lista_de_dois() {
    let conteudo = format!(
        "{}{}",
        ancora("7", "31", "amarelo", "Texto sintético", "Pendente"),
        ancora("7", "32", "azul", "", "Revis&#227;o")
    );
    assert_eq!(
        marcadores(&conteudo),
        json!([
            {"nome": "Pendente", "cor": "amarelo", "texto": "Texto sintético", "id": "31", "rotulo": "31#Pendente"},
            {"nome": "Revisão", "cor": "azul", "texto": null, "id": "32", "rotulo": "32#Revisão"}
        ])
    );
}

#[test]
fn celula_vazia_e_lista_vazia_e_nao_null() {
    assert_eq!(marcadores("&nbsp;"), json!([]));
    assert_eq!(marcadores(""), json!([]));
}

#[test]
fn tooltip_le_o_literal_com_escapes_e_entidades() {
    let conteudo = ancora(
        "7",
        "40",
        "vermelho",
        "linha um\\nlinha \\'dois\\' &amp; tr&#234;s",
        "Urgente",
    );
    assert_eq!(
        marcadores(&conteudo)[0]["texto"],
        "linha um linha 'dois' & três"
    );
}

#[test]
fn sem_tooltip_ou_sem_fragmento_publica_null() {
    let sem_dica = ancora("7", "41", "verde", "x", "M1")
        .replace(" onmouseover=\"return infraTooltipMostrar('x');\"", "");
    let sem_id = ancora("7", "41", "verde", "", "M2").replace("#ID-41", "");
    assert_eq!(
        marcadores(&format!("{sem_dica}{sem_id}")),
        json!([
            {"nome": "M1", "cor": "verde", "texto": null, "id": "41", "rotulo": "41#M1"},
            {"nome": "M2", "cor": "verde", "texto": null, "id": null, "rotulo": null}
        ])
    );
}

#[test]
fn ancora_fora_da_forma_observada_e_recusa_tipada() {
    let boa = ancora("7", "31", "azul", "", "M");
    let ruins = [
        boa.replace("acao=andamento_marcador_gerenciar", "acao=outra_acao"),
        boa.replace("id_procedimento=7&", "id_procedimento=8&"),
        boa.replace("svg/marcador_azul.svg", "svg/outro.svg"),
        boa.replace("marcador_azul", "marcador_Azul"),
        boa.replace("#ID-31", "#X-31"),
        boa.replace("#ID-31", "#ID-3a"),
        boa.replace(
            "return infraTooltipMostrar('');",
            "return infraTooltipMostrar('a','b');",
        ),
        boa.replace("return infraTooltipMostrar('');", "alert('x');"),
        boa.replace("<div>M</div>", ""),
        boa.replace("<div>M</div>", "<div></div>"),
        boa.replace(
            "class=\"imagemStatus\"/>",
            "class=\"imagemStatus\"/><img src=\"svg/x.svg\"/>",
        ),
        boa.replace("class=\"ancMarcador\" ", ""),
    ];
    for ruim in ruins {
        assert_eq!(
            com_marcadores(&ruim),
            Err(ErroDetalhada::Html(ErroHtml::valor(
                "marcador_da_celula_detalhada_fora_da_forma_observada"
            ))),
            "{ruim}"
        );
    }
    let erro = ErroDetalhada::Html(ErroHtml::valor(
        "marcador_da_celula_detalhada_fora_da_forma_observada",
    ));
    assert_eq!(
        crate::erro_publico::folha(&erro),
        "marcador_da_celula_detalhada_fora_da_forma_observada"
    );
}

#[test]
fn texto_fora_das_ancoras_e_recusa_e_nao_vira_marcador() {
    let boa = ancora("7", "31", "azul", "", "M");
    for conteudo in [
        format!("{boa} solto"),
        "PendenteRevisão".to_owned(),
        format!("<span>x</span>{boa}"),
    ] {
        assert_eq!(
            com_marcadores(&conteudo),
            Err(ErroDetalhada::TextoForaDosMarcadoresNaCelula),
            "{conteudo}"
        );
    }
    assert_eq!(
        crate::erro_publico::folha(&ErroDetalhada::TextoForaDosMarcadoresNaCelula),
        "texto_fora_dos_marcadores_na_celula"
    );
}

#[test]
fn colheita_da_pagina_d_com_marcadores_continua_dando_o_link() {
    let conteudo = format!(
        "{}{}",
        ancora("7", "31", "amarelo", "", "A"),
        ancora("7", "32", "azul", "", "B")
    );
    let html = tabela(&[rotulo("Marcadores")], &[&celula(&conteudo)]);
    let linhas = linhas_com_link(&html).unwrap();
    assert_eq!(linhas.len(), 1);
    assert_eq!(linhas[0].id, "7");
    assert!(
        linhas[0]
            .href
            .as_deref()
            .is_some_and(|h| h.contains("acao=procedimento_trabalhar"))
    );
}
