//! A tela "Conclusão de Processo" (forma `tela`, SEI 4.1 e 5.0) e a
//! resposta das duas formas (contracts/cli/processo-concluir.md).
//!
//! Evidência (cassettes públicas do Todos, UFRJ 4.1.5, PEN2 4.1.5, GO
//! 5.0.3, PEN3 e PEN4 5.0.4, telas idênticas): `form#frmDesentranharDocumento`
//! `method="post"`; ocultos `hdnInfraTipoPagina`, `hdnIdProtocolos` (o id) e
//! `hdnIdProcedimentoDestino`; `sbmSalvar`; `rdoConcluir` `S` marcado e `V`;
//! a reabertura programada vazia. O navegador envia `rdoConcluir=S` e os
//! dois textos vazios (teste do Todos que mediu o envio na UFRJ); o SEI
//! responde 302 para `arvore_visualizar` (ou `principal`), da conclusão do
//! mesmo processo.
use crate::arvore_processo::ACAO_DE_CONCLUIR;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, ResultadoDoEnvio, consulta_do_controlador, localizar_formulario,
};
use crate::html_atributos::{atributo_presente, valor_do_atributo};

pub(crate) const FORMULARIO_DE_CONCLUSAO: &str = "frmDesentranharDocumento";
pub(crate) const BOTAO_SALVAR: &str = "sbmSalvar";
const OCULTOS: [&str; 3] = [
    "hdnInfraTipoPagina",
    "hdnIdProtocolos",
    "hdnIdProcedimentoDestino",
];
const OPCAO: &str = "rdoConcluir";
const SOMENTE_CONCLUIR: &str = "S";
const PRAZO: &str = "rdoPrazoReaberturaProgramada";
const DATA: &str = "txtPrazoReaberturaProgramada";
const DIAS: &str = "txtDiasReaberturaProgramada";
const UTEIS: &str = "chkSinDiasUteisReaberturaProgramada";
const PROCESSOS: &str = "selProcedimentos";

/// Salvar a tela de conclusão: janela sem barra da unidade, aberta pelo
/// link da barra da árvore da tela do processo conferida.
pub(crate) const CONCLUIR_PROCESSO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: ACAO_DE_CONCLUIR,
    formulario: FORMULARIO_DE_CONCLUSAO,
    envio: EnvioDoFormulario::BotaoDeEnvio(BOTAO_SALVAR),
    campos_ocultos_exigidos: &OCULTOS,
    campos_ocultos_reescritos: &[],
    campos_usuario: &[OPCAO, DATA, DIAS],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::AndamentoAbertoNaUnidade,
    },
};

/// Os pares do usuário: "Somente concluir" e os textos vazios.
pub(crate) const CAMPOS_DO_ENVIO: [(&str, &str); 3] =
    [(OPCAO, SOMENTE_CONCLUIR), (DATA, ""), (DIAS, "")];

/// Confere a tela servida antes de qualquer POST: lista fechada de
/// controles enviáveis, o processo, "Somente concluir" marcado e nada da
/// reabertura programada preenchido.
pub(crate) fn conferir_tela_de_conclusao(
    html: &str,
    id: &str,
) -> Result<(), super::erro::ErroTelaDeConclusao> {
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| ErroHtml::html("tela_de_conclusao_nao_interpretada"))?;
    let parser = dom.parser();
    let form = localizar_formulario(&dom, FORMULARIO_DE_CONCLUSAO)
        .map_err(|_| ErroHtml::ausente("formulario_de_conclusao_ausente"))?;
    let mut somente_marcado = false;
    let mut outra_marcada = false;
    let mut preenchida = false;
    for tag in form
        .children()
        .all(parser)
        .iter()
        .filter_map(tl::Node::as_tag)
    {
        let elemento = tag.name().as_utf8_str().to_ascii_lowercase();
        if elemento == "option" {
            preenchida |= presente(tag, "selected")?;
            continue;
        }
        if !matches!(
            elemento.as_str(),
            "input" | "button" | "select" | "textarea"
        ) || presente(tag, "disabled")?
        {
            continue;
        }
        let Some(nome) = valor(tag, "name")?.filter(|n| !n.is_empty()) else {
            continue;
        };
        let padrao = if elemento == "button" {
            "submit"
        } else {
            "text"
        };
        let tipo =
            valor(tag, "type")?.map_or_else(|| padrao.to_owned(), |t| t.to_ascii_lowercase());
        let marcado = presente(tag, "checked")?;
        let servido = valor(tag, "value")?.unwrap_or_default();
        match (elemento.as_str(), tipo.as_str(), nome.as_str()) {
            ("input", "hidden", "hdnIdProtocolos") if servido != id => {
                return Err(super::erro::ErroTelaDeConclusao::TelaDeConclusaoDeOutroProcesso);
            }
            ("input", "hidden", n) if OCULTOS.contains(&n) => {}
            // `button`/`reset` não participam do envio.
            ("input" | "button", "button" | "reset", _)
            | ("input" | "button", "submit", BOTAO_SALVAR)
            | ("select", _, PROCESSOS) => {}
            ("input", "radio", OPCAO) => {
                if marcado && servido == SOMENTE_CONCLUIR {
                    somente_marcado = true;
                } else if marcado {
                    outra_marcada = true;
                }
            }
            ("input", "radio", PRAZO) | ("input", "checkbox", UTEIS) => preenchida |= marcado,
            ("input", "text", DATA | DIAS) => preenchida |= !servido.is_empty(),
            _ => return Err(ErroHtml::valor("campo_nao_previsto_na_tela_de_conclusao").into()),
        }
    }
    if !somente_marcado || outra_marcada {
        return Err(super::erro::ErroTelaDeConclusao::OpcaoSomenteConcluirNaoMarcada);
    }
    if preenchida {
        return Err(super::erro::ErroTelaDeConclusao::TelaDeConclusaoComReaberturaPreenchida);
    }
    Ok(())
}

fn presente(tag: &tl::HTMLTag<'_>, nome: &str) -> Result<bool, super::erro::ErroTelaDeConclusao> {
    atributo_presente(tag, nome).map_err(|e| {
        super::erro::ErroTelaDeConclusao::Html(ErroHtml::atributo(
            "tela_de_conclusao_nao_interpretada",
            e,
        ))
    })
}

fn valor(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, super::erro::ErroTelaDeConclusao> {
    valor_do_atributo(tag, nome).map_err(|e| {
        super::erro::ErroTelaDeConclusao::Html(ErroHtml::atributo(
            "tela_de_conclusao_nao_interpretada",
            e,
        ))
    })
}

/// O 302 do POST é o esperado: `controlador.php?acao=arvore_visualizar`
/// ou `acao=principal` (relativo, como gravado, ou no host e na pasta do
/// POST: ver `consulta_do_controlador`), com
/// `acao_origem=procedimento_concluir` e o id, cada chave uma vez.
pub(crate) fn redirecionamento_esperado(
    localizacao: Option<&str>,
    url_do_envio: &str,
    id: &str,
) -> bool {
    let Some(consulta) = localizacao.and_then(|l| consulta_do_controlador(l, url_do_envio)) else {
        return false;
    };
    let valores = |chave: &str| -> Vec<&str> {
        consulta
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(k, _)| *k == chave)
            .map(|(_, v)| v)
            .collect()
    };
    matches!(
        valores("acao").as_slice(),
        ["arvore_visualizar" | "principal"]
    ) && valores("acao_origem") == [ACAO_DE_CONCLUIR]
        && valores("id_procedimento") == [id]
}
