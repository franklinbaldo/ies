//! Tela `procedimento_trabalhar`: a unidade publicada (`#lnkInfraUnidade`)
//! e o `src` assinado do único `iframe#ifrArvore`, conferido e resolvido.
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::{
    ConsultaDoControlador, ErroConsultaDoControlador, ErroParametroDaUrl,
};
use crate::instalacao::Instalacao;

/// URL absoluta da árvore, assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDaArvore(String);

impl EnderecoDaArvore {
    /// Só para URLs já conferidas por [`super::endereco`].
    pub(super) fn conferido(url: String) -> Self {
        Self(url)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDaArvore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDaArvore(<redigido>)")
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDoProcesso {
    pub(crate) unidade: String,
    pub(crate) arvore: EnderecoDaArvore,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDoProcesso {
    UnidadeDoControleIlegivel(#[source] crate::erro_web::ErroCaixa),
    UnidadeDaTelaDiferenteDaSolicitada,
    EnderecoDaArvoreDeOutroProcesso,
    /// Tela aberta por número SEI: a árvore não seleciona o documento pedido.
    EnderecoDaArvoreDeOutroDocumento,
    /// O SEI exige validar o acesso (senha, modal `usuario_validar_acesso`)
    /// antes de servir a árvore, que vem com `src` vazio (GO 5.0.3, live de
    /// 10/11/2026). A validação em si não é feita pelo IES.
    ProcessoExigeValidacaoDeAcesso,
    ArvoreAusenteNaTelaDoProcesso,
    ArvoresRepetidasNaTelaDoProcesso,
    EnderecoDaArvoreNaoReconhecido,
    /// A consulta do `src` da árvore não se deixa ler em pares.
    ConsultaDaArvoreIlegivel(#[source] ErroConsultaDoControlador),
    /// O `src` da tela pesquisada não traz exatamente um `id_procedimento`.
    IdDoProcessoNaArvore(#[source] ErroParametroDaUrl),
    IdentificacaoDaUnidadeAusenteNaTelaDoProcesso,
}

/// Lê a tela servida (a URL já foi conferida por quem a pediu). `unidade`
/// pedida (`--unidade`) é comparada com espaços normalizados.
pub(crate) fn ler_tela_do_processo(
    instalacao: &Instalacao,
    html: &crate::tela_html::HtmlServido,
    id: &str,
    unidade: Option<&str>,
) -> Result<TelaDoProcesso, ErroTelaDoProcesso> {
    let (publicada, src) = unidade_e_src(html, unidade)?;
    let arvore = conferir_endereco(instalacao, &src, id)?;
    Ok(TelaDoProcesso {
        unidade: publicada,
        arvore,
    })
}

/// A tela aberta pela pesquisa rápida de um número SEI (forma das oito
/// cassettes `documento-numero-e-busca-*` e `documento-propriedades-unb`):
/// o `src` da árvore traz `id_procedimento` do processo e `id_documento`
/// igual ao protocolo pesquisado. Devolve a tela e o id do processo.
pub(crate) fn ler_tela_do_documento(
    instalacao: &Instalacao,
    html: &crate::tela_html::HtmlServido,
    id_documento: &str,
    unidade: Option<&str>,
) -> Result<(TelaDoProcesso, String), ErroTelaDoProcesso> {
    let (publicada, src) = unidade_e_src(html, unidade)?;
    let consulta = ConsultaDoControlador::da_url(src.split_once('?').map(|(_, c)| c))
        .map_err(ErroTelaDoProcesso::ConsultaDaArvoreIlegivel)?;
    if consulta.valores("id_documento").collect::<Vec<_>>() != [id_documento] {
        return Err(ErroTelaDoProcesso::EnderecoDaArvoreDeOutroDocumento);
    }
    let id_procedimento = consulta
        .unico("id_procedimento")
        .map_err(ErroTelaDoProcesso::IdDoProcessoNaArvore)?;
    let arvore = conferir_endereco(instalacao, &src, id_procedimento)?;
    Ok((
        TelaDoProcesso {
            unidade: publicada,
            arvore,
        },
        id_procedimento.to_owned(),
    ))
}

/// A unidade publicada (conferida com `--unidade`) e o `src` decodificado
/// do único `iframe#ifrArvore`.
fn unidade_e_src(
    html: &crate::tela_html::HtmlServido,
    unidade: Option<&str>,
) -> Result<(String, String), ErroTelaDoProcesso> {
    let dom = crate::tela_html::arvore(html);
    let publicada = crate::controle_contexto::unidade_html(&dom)
        .map_err(ErroTelaDoProcesso::UnidadeDoControleIlegivel)?
        .ok_or(ErroTelaDoProcesso::IdentificacaoDaUnidadeAusenteNaTelaDoProcesso)?;
    if let Some(pedida) = unidade
        && pedida.split_whitespace().collect::<Vec<_>>().join(" ") != publicada
    {
        return Err(ErroTelaDoProcesso::UnidadeDaTelaDiferenteDaSolicitada);
    }
    let iframes = dom.com_id("ifrArvore").collect::<Vec<_>>();
    let iframe = match iframes.as_slice() {
        [] => return Err(ErroTelaDoProcesso::ArvoreAusenteNaTelaDoProcesso),
        [um] => *um,
        _ => return Err(ErroTelaDoProcesso::ArvoresRepetidasNaTelaDoProcesso),
    };
    if iframe.nome() != "iframe" {
        return Err(ErroTelaDoProcesso::ArvoreAusenteNaTelaDoProcesso);
    }
    let src = iframe
        .atributo("src")
        .map(str::to_owned)
        .ok_or(ErroTelaDoProcesso::EnderecoDaArvoreNaoReconhecido)?;
    // Servido com `&` cru (UnB e GO); `&amp;` também é o mesmo endereço.
    if src.is_empty() && html.contains("controlador.php?acao=usuario_validar_acesso&") {
        return Err(ErroTelaDoProcesso::ProcessoExigeValidacaoDeAcesso);
    }
    Ok((publicada, src))
}

/// O `src` da árvore: `acao=procedimento_visualizar`, conferido pela regra
/// comum de [`super::endereco`].
fn conferir_endereco(
    instalacao: &Instalacao,
    src: &str,
    id: &str,
) -> Result<EnderecoDaArvore, ErroTelaDoProcesso> {
    use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
    conferir_endereco_relativo(instalacao, src, AcaoDoSei::ProcedimentoVisualizar, id)
        .map(EnderecoDaArvore)
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => ErroTelaDoProcesso::EnderecoDaArvoreNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => {
                ErroTelaDoProcesso::EnderecoDaArvoreDeOutroProcesso
            }
        })
}
