//! Tela `procedimento_trabalhar`: a unidade publicada (`#lnkInfraUnidade`)
//! e o `src` assinado do único `iframe#ifrArvore`, conferido e resolvido.
use crate::html_atributos::{nos_com_id, valor_do_atributo};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroTelaDoProcesso {
    HtmlDaTelaDoProcessoNaoInterpretado,
    IdentificacaoDaUnidadeAusenteNaTelaDoProcesso,
    UnidadeDaTelaDiferenteDaSolicitada,
    ArvoreAusenteNaTelaDoProcesso,
    ArvoresRepetidasNaTelaDoProcesso,
    EnderecoDaArvoreNaoReconhecido,
    EnderecoDaArvoreDeOutroProcesso,
    /// Tela aberta por número SEI: a árvore não seleciona o documento pedido.
    EnderecoDaArvoreDeOutroDocumento,
}

impl ErroTelaDoProcesso {
    pub(crate) fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
}

/// Lê a tela servida (a URL já foi conferida por quem a pediu). `unidade`
/// pedida (`--unidade`) é comparada com espaços normalizados.
pub(crate) fn ler_tela_do_processo(
    instalacao: &Instalacao,
    html: &str,
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
    html: &str,
    id_documento: &str,
    unidade: Option<&str>,
) -> Result<(TelaDoProcesso, String), ErroTelaDoProcesso> {
    use ErroTelaDoProcesso as E;
    let (publicada, src) = unidade_e_src(html, unidade)?;
    let valores = |chave: &str| -> Vec<String> {
        src.split_once('?')
            .map_or("", |(_, consulta)| consulta)
            .split('&')
            .filter_map(|par| par.split_once('='))
            .filter(|(nome, _)| *nome == chave)
            .map(|(_, valor)| valor.to_owned())
            .collect()
    };
    if valores("id_documento") != [id_documento] {
        return Err(E::EnderecoDaArvoreDeOutroDocumento);
    }
    let [id_procedimento] = valores("id_procedimento")
        .try_into()
        .map_err(|_| E::EnderecoDaArvoreNaoReconhecido)?;
    let arvore = conferir_endereco(instalacao, &src, &id_procedimento)?;
    Ok((
        TelaDoProcesso {
            unidade: publicada,
            arvore,
        },
        id_procedimento,
    ))
}

/// A unidade publicada (conferida com `--unidade`) e o `src` decodificado
/// do único `iframe#ifrArvore`.
fn unidade_e_src(
    html: &str,
    unidade: Option<&str>,
) -> Result<(String, String), ErroTelaDoProcesso> {
    use ErroTelaDoProcesso as E;
    let dom = tl::parse(html, tl::ParserOptions::default())
        .map_err(|_| E::HtmlDaTelaDoProcessoNaoInterpretado)?;
    let publicada = crate::controle_contexto::unidade_html(&dom)
        .map_err(|_| E::HtmlDaTelaDoProcessoNaoInterpretado)?
        .ok_or(E::IdentificacaoDaUnidadeAusenteNaTelaDoProcesso)?;
    if let Some(pedida) = unidade
        && pedida.split_whitespace().collect::<Vec<_>>().join(" ") != publicada
    {
        return Err(E::UnidadeDaTelaDiferenteDaSolicitada);
    }
    let iframes =
        nos_com_id(&dom, "ifrArvore").map_err(|_| E::HtmlDaTelaDoProcessoNaoInterpretado)?;
    let iframe = match iframes.as_slice() {
        [] => return Err(E::ArvoreAusenteNaTelaDoProcesso),
        [um] => um
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(E::HtmlDaTelaDoProcessoNaoInterpretado)?,
        _ => return Err(E::ArvoresRepetidasNaTelaDoProcesso),
    };
    if iframe.name() != "iframe" {
        return Err(E::ArvoreAusenteNaTelaDoProcesso);
    }
    let src = valor_do_atributo(iframe, "src")
        .map_err(|_| E::HtmlDaTelaDoProcessoNaoInterpretado)?
        .ok_or(E::EnderecoDaArvoreNaoReconhecido)?;
    // Servido com `&` cru (UnB e GO); `&amp;` também é o mesmo endereço.
    let src = crate::formulario_web::decodificar_atributo(&src)
        .ok_or(E::EnderecoDaArvoreNaoReconhecido)?;
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
    conferir_endereco_relativo(instalacao, src, "procedimento_visualizar", id)
        .map(EnderecoDaArvore)
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => ErroTelaDoProcesso::EnderecoDaArvoreNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => {
                ErroTelaDoProcesso::EnderecoDaArvoreDeOutroProcesso
            }
        })
}
