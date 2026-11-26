//! Travessia da consulta de acervo com o envio nativo, sem retry de POST.
use super::{
    ErroAcervo, ProcessoDoAcervo, conferir_pagina, detalhe, formulario_do_acervo,
    total_do_resultado,
};
use crate::acao_do_sei::AcaoDoSei;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, ErroSubmissaoFormulario,
    EstadoDeSessao, OperacaoDeFormulario, ResultadoDoEnvio, SubmissaoDerivada, TelaServida,
    derivar_submissao,
};
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::tela_html::{HtmlServido, arvore};
use crate::transporte::envio_post::ler_tela_via;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::unidade_retornos::DataCivil;
use crate::visao_do_controle::{EnvioContado, EnvioDoControle, EnvioHttps, TelaAtual};

/// Reutiliza o formulário da página atual, sem fabricar action ou hash.
/// O índice é zero-based; o corpo original não foi persistido pelo gravador.
pub(super) fn formulario_da_pagina(
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    pagina: usize,
) -> Result<SubmissaoDerivada, ErroSubmissaoFormulario> {
    const PAGINAR: OperacaoDeFormulario = OperacaoDeFormulario {
        acao: crate::acao_do_sei::AcaoDoSei::EstatisticasDetalharUnidade,
        formulario: "frmEstatisticasLista",
        envio: EnvioDoFormulario::SubmitPorScript,
        campos_ocultos_exigidos: &["hdnInfraPaginaAtual"],
        campos_ocultos_reescritos: &["hdnInfraPaginaAtual"],
        campos_usuario: &[],
        resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
        unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
        efeito: EfeitoAvaliado::AlteraEstadoDaSessao {
            o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
        },
    };
    derivar_submissao(
        &PAGINAR,
        instalacao,
        tela,
        Some(unidade),
        &[("hdnInfraPaginaAtual", &pagina.to_string())],
    )
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(super) enum ErroEnvioPagina {
    Submissao(#[from] ErroSubmissaoFormulario),
    Redirecionamento,
}

/// Conserva a URL do POST e o MIME da resposta para a próxima derivação.
/// Reutiliza todas as verificações do envio comum, sem repetir a requisição.
pub(super) fn enviar_pagina_via<R: std::io::Read>(
    submissao: SubmissaoDerivada,
    cookie: &crate::transporte::sessao::CookieDaSessao,
    enviar: impl FnOnce(
        SubmissaoDerivada,
        &str,
    ) -> Result<
        crate::formulario_web::RespostaDoPost<R>,
        crate::transporte::FalhaTransporte,
    >,
) -> Result<(String, String, HtmlServido), ErroEnvioPagina> {
    use crate::formulario_web::{RespostaDoEnvio, enviar_submissao_via};
    let url = submissao.url().to_owned();
    let mut mime = String::new();
    let resposta = enviar_submissao_via(submissao, cookie, |post, cookie| {
        let resposta = enviar(post, cookie)?;
        mime.clone_from(&resposta.resposta.mime);
        Ok(resposta)
    })?;
    match resposta {
        RespostaDoEnvio::Html(html) => Ok((url, mime, html)),
        RespostaDoEnvio::Redirecionamento(_) => Err(ErroEnvioPagina::Redirecionamento),
    }
}

fn link_do_menu(instalacao: &Instalacao, html: &HtmlServido) -> Result<String, ErroAcervo> {
    use crate::arvore_processo::endereco::conferir_endereco_relativo_por;
    use crate::consulta_do_controlador::menciona_acao;
    let acao = AcaoDoSei::GerarEstatisticasUnidade;
    let dom = arvore(html);
    let mut links = dom
        .elementos("a")
        .filter_map(|a| a.atributo("href"))
        .filter(|href| menciona_acao(href, acao));
    let href = links
        .next()
        .ok_or(ErroAcervo::MenuDeEstatisticasNaoReconhecido)?;
    if links.next().is_some() {
        return Err(ErroAcervo::MenuDeEstatisticasNaoReconhecido);
    }
    conferir_endereco_relativo_por(instalacao, href, acao, "acao", acao.como_na_url())
        .map_err(|_| ErroAcervo::MenuDeEstatisticasNaoReconhecido)
}

fn tela_servida(tela: &TelaAtual) -> Result<TelaServida<'_>, ErroAcervo> {
    Ok(TelaServida {
        url: &tela.url,
        html: &tela.html,
        codificacao: codificacao_da_tela(&tela.mime)
            .ok_or(ErroAcervo::CharsetDaTelaNaoSuportado)?,
    })
}

fn get<V: EnvioDoControle>(
    envio: &mut V,
    sessao: &ReferenciasDaSessao,
    url: String,
) -> Result<TelaAtual, crate::transporte::FalhaTransporte> {
    // O chamador fornece somente o link conferido na mesma instalação.
    let resposta = envio.get(&url, sessao.cookie().valor()?)?;
    let mime = resposta.mime.clone();
    let html = crate::transporte::decodificar_resposta(resposta)?;
    Ok(TelaAtual { url, html, mime })
}

fn post<V: EnvioDoControle>(
    envio: &mut V,
    sessao: &ReferenciasDaSessao,
    submissao: SubmissaoDerivada,
) -> Result<TelaAtual, ErroEnvioPagina> {
    let (url, mime, html) = enviar_pagina_via(submissao, sessao.cookie(), |post, cookie| {
        envio.post(post, cookie)
    })?;
    Ok(TelaAtual { url, html, mime })
}

fn ler_via<V: EnvioDoControle>(
    sessao: &ReferenciasDaSessao,
    unidade_esperada: Option<&str>,
    data: DataCivil,
    envio: &mut V,
) -> anyhow::Result<(String, Vec<ProcessoDoAcervo>)> {
    let controle = ler_tela_via(
        sessao.url_do_controle(),
        sessao.cookie(),
        sessao.origem(),
        |url, cookie| envio.get(url, cookie),
    )?;
    let unidade = crate::controle_reconhecido::unidade_do_controle(&controle.html)?;
    if unidade_esperada.is_some_and(|esperada| esperada != unidade) {
        return Err(ErroAcervo::UnidadeDiferenteDaSolicitada.into());
    }
    let inst = &controle.instalacao;
    let url = link_do_menu(inst, &controle.html)?;
    let tela = get(envio, sessao, url)?;
    let consulta = formulario_do_acervo(inst, &tela_servida(&tela)?, &unidade, data)?;
    let resultado = post(envio, sessao, consulta)?;
    let (total, url) = total_do_resultado(&resultado.html, inst)?;
    let mut tela = get(envio, sessao, url)?;
    let mut processos = Vec::new();
    let mut indice = 0;
    loop {
        let pagina = detalhe(&tela.html)?;
        conferir_pagina(&pagina, total, processos.len(), indice == 0)?;
        processos.extend(pagina.processos);
        if processos.len() == total {
            return Ok((unidade, processos));
        }
        indice += 1;
        let submissao = formulario_da_pagina(inst, &tela_servida(&tela)?, &unidade, indice)?;
        tela = post(envio, sessao, submissao)?;
    }
}

pub(crate) fn listar(
    unidade: Option<&str>,
    json_mode: bool,
    data: DataCivil,
) -> anyhow::Result<()> {
    let arquivos = crate::configuracao::arquivos_de_acesso()?;
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let mut envio = EnvioContado::novo(EnvioHttps);
    // Não envolver em relogin que recomeçaria os POSTs de paginação.
    let (unidade, processos) = ler_via(&sessao, unidade, data, &mut envio)?;
    let total = processos.len();
    let distintos = processos
        .iter()
        .map(|p| p.processo.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();
    if json_mode {
        println!(
            "{}",
            serde_json::json!({
                "unidade": unidade, "data": data.to_string(), "origem": "web",
                "total_declarado": total, "linhas_observadas": total,
                "nups_distintos": distintos, "processos": processos,
                "requisicoes": envio.pedidos.total(),
            })
        );
    } else {
        println!(
            "Acervo da unidade {unidade} em {data}: {total} linhas, {distintos} NUPs distintos."
        );
        for processo in processos {
            println!(
                "{} | {} | {} | {} | {} | {}",
                processo.processo,
                processo.tipo,
                processo.orgao,
                processo.unidade,
                processo.mes,
                processo.ano
            );
        }
    }
    Ok(())
}

#[test]
fn menu_exige_link_unico_assinado_da_instalacao() {
    use crate::formulario_web::tests::{HASH_TELA, instalacao};
    let inst = instalacao();
    let link = format!(
        "<a href='controlador.php?acao=gerar_estatisticas_unidade&amp;infra_hash={HASH_TELA}'>Estatísticas</a>"
    );
    assert!(link_do_menu(&inst, &HtmlServido::de_teste(&link)).is_ok());
    for html in [
        String::new(),
        format!("{link}{link}"),
        link.replace("controlador.php?", "https://fora.example/controlador.php?"),
        link.replace(HASH_TELA, "x"),
    ] {
        assert!(link_do_menu(&inst, &HtmlServido::de_teste(&html)).is_err());
    }
}
