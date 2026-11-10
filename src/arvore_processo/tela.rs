//! Tela `procedimento_trabalhar`: a unidade publicada (`#lnkInfraUnidade`)
//! e o `src` assinado do único `iframe#ifrArvore`, conferido e resolvido.
use crate::erro_html::ErroHtml;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroTelaDoProcesso {
    UnidadeDaTelaDiferenteDaSolicitada,
    EnderecoDaArvoreDeOutroProcesso,
    /// Tela aberta por número SEI: a árvore não seleciona o documento pedido.
    EnderecoDaArvoreDeOutroDocumento,
    /// O SEI exige validar o acesso (senha, modal `usuario_validar_acesso`)
    /// antes de servir a árvore, que vem com `src` vazio (GO 5.0.3, live de
    /// 10/11/2026). A validação em si não é feita pelo IES.
    ProcessoExigeValidacaoDeAcesso,
    Html(#[from] ErroHtml),
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
        .map_err(|_| ErroHtml::valor("endereco_da_arvore_nao_reconhecido"))?;
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
        .map_err(|_| ErroHtml::html("html_da_tela_do_processo_nao_interpretado"))?;
    let publicada = crate::controle_contexto::unidade_html(&dom)
        .map_err(|_| ErroHtml::html("html_da_tela_do_processo_nao_interpretado"))?
        .ok_or(ErroHtml::ausente(
            "identificacao_da_unidade_ausente_na_tela_do_processo",
        ))?;
    if let Some(pedida) = unidade
        && pedida.split_whitespace().collect::<Vec<_>>().join(" ") != publicada
    {
        return Err(E::UnidadeDaTelaDiferenteDaSolicitada);
    }
    let iframes = nos_com_id(&dom, "ifrArvore")
        .map_err(|e| ErroHtml::atributo("html_da_tela_do_processo_nao_interpretado", e))?;
    let iframe = match iframes.as_slice() {
        [] => return Err(ErroHtml::ausente("arvore_ausente_na_tela_do_processo").into()),
        [um] => um
            .get(dom.parser())
            .and_then(tl::Node::as_tag)
            .ok_or(ErroHtml::html("html_da_tela_do_processo_nao_interpretado"))?,
        _ => return Err(ErroHtml::repetido("arvores_repetidas_na_tela_do_processo").into()),
    };
    if iframe.name() != "iframe" {
        return Err(ErroHtml::ausente("arvore_ausente_na_tela_do_processo").into());
    }
    let src = valor_do_atributo(iframe, "src")
        .map_err(|e| ErroHtml::atributo("html_da_tela_do_processo_nao_interpretado", e))?
        .ok_or(ErroHtml::valor("endereco_da_arvore_nao_reconhecido"))?;
    // Servido com `&` cru (UnB e GO); `&amp;` também é o mesmo endereço.
    let src = crate::formulario_web::decodificar_atributo(&src)
        .ok_or(ErroHtml::valor("endereco_da_arvore_nao_reconhecido"))?;
    if src.is_empty() && html.contains("controlador.php?acao=usuario_validar_acesso&") {
        return Err(E::ProcessoExigeValidacaoDeAcesso);
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
    conferir_endereco_relativo(instalacao, src, "procedimento_visualizar", id)
        .map(EnderecoDaArvore)
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                ErroTelaDoProcesso::Html(ErroHtml::valor("endereco_da_arvore_nao_reconhecido"))
            }
            EnderecoRecusado::DeOutroProcesso => {
                ErroTelaDoProcesso::EnderecoDaArvoreDeOutroProcesso
            }
        })
}
