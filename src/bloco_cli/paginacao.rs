//! Paginação da lista de blocos internos por formulário observado, sem JS.
use super::FalhaLeitura;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDeSessao, OperacaoDeFormulario,
    ResultadoDoEnvio,
};

/// Navegação observada na cassette GO bloco-interno-ciclo, passo 11 GET
/// seguido de passo 12 POST: índice zero-based 1 -> faixa 51 a 100.
/// O corpo do POST original nao foi preservado, logo a derivacao segue
/// estritamente os controles que a resposta HTML publicou.
pub(super) const PAGINAR_BLOCOS: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "bloco_interno_listar",
    formulario: "frmBlocoLista",
    envio: EnvioDoFormulario::SubmitPorScript,
    campos_ocultos_exigidos: &["hdnInfraPaginaAtual", "hdnInfraTipoPagina"],
    campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
    campos_usuario: &["selInfraPaginacaoSuperior", "selInfraPaginacaoInferior"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
        o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
    },
};

/// Prova que o formulário serviu exatamente a opção de destino. Sem executar JS.
pub(super) fn conferir_indice_da_pagina(html: &str, indice: &str) -> Result<(), FalhaLeitura> {
    use crate::html_atributos::valor_do_atributo;
    let ilegivel = || FalhaLeitura::Html(ErroHtml::valor("paginador_de_blocos_invalido"));
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| ilegivel())?;
    let form = crate::formulario_web::localizar_formulario(&dom, "frmBlocoLista")
        .map_err(FalhaLeitura::Submissao)?;
    let mut achados = 0;
    for no in form
        .query_selector(dom.parser(), "select")
        .into_iter()
        .flatten()
    {
        let tag = no
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or_else(ilegivel)?;
        if valor_do_atributo(tag, "name")
            .map_err(|_| ilegivel())?
            .as_deref()
            != Some("selInfraPaginacaoSuperior")
        {
            continue;
        }
        achados += 1;
        let opcoes = tag
            .query_selector(dom.parser(), "option")
            .ok_or_else(ilegivel)?;
        let valores = opcoes
            .filter_map(|h| h.get(dom.parser()).and_then(tl::Node::as_tag))
            .map(|o| valor_do_atributo(o, "value").map_err(|_| ilegivel()))
            .collect::<Result<Vec<_>, _>>()?;
        if valores
            .iter()
            .filter(|v| v.as_deref() == Some(indice))
            .count()
            != 1
        {
            return Err(ilegivel());
        }
    }
    if achados != 1 {
        return Err(ilegivel());
    }
    Ok(())
}
