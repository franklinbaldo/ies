//! `ies bloco incluir BLOCO --documento DOC` (#381; contracts/cli/bloco-incluir.md):
//! prévia da inclusão de documento em bloco de assinatura. Abre o documento
//! até a árvore, segue o `bloco_escolher` publicado na barra do documento e
//! confere no `frmBlocoEscolher` que o bloco e o documento são oferecidos;
//! o pertencimento anterior vem do detalhe do bloco, a mesma leitura de
//! `ies bloco N`. Nenhum POST: a confirmação é a próxima fatia (BDD-381-02).
use crate::arvore_processo::{ErroBlocoNaBarra, ler_bloco_escolher_da_barra};
use crate::bloco::TipoBloco;
use crate::bloco_cli::{FalhaLeitura, Leitura, ler_tipado_via};
use crate::documento::NumeroDocumento;
use crate::erro_html::ErroHtml;
use crate::formulario_web::{
    ConferenciaDaUnidade, EfeitoAvaliado, EnvioDoFormulario, EstadoDoProcesso,
    OperacaoDeFormulario, RespostaDoEnvio, ResultadoDoEnvio, TelaServida, consulta_do_controlador,
    decodificar_atributo, derivar_submissao, enviar_submissao_via,
};
use crate::html_atributos::atributos;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::pagina_do_controle::codificacao_da_tela;
use crate::processo_concluir::escrita::{
    Contador, CookieIlegivel, FalhaDaTela, ler_cookie, obter_tela,
};
use crate::processo_obter::{
    DocumentoNaArvoreAberto, EnvioDoProcesso, ErroDocumentoObter, PedidoDoAnexo,
    PedidoDoDocumento, SessaoDoProcesso, abrir_documento_na_arvore,
};
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::EnvioDoControle as _;

/// Blocos e documentos oferecidos pelo `frmBlocoEscolher`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TelaDeEscolha {
    /// Números dos blocos de `selBloco` (sem a opção vazia).
    pub(crate) blocos: Vec<String>,
    /// Número SEI (`title`) de cada caixa `chkDocumentosItemN`.
    pub(crate) documentos: Vec<String>,
}

/// Prévia da inclusão: o que foi observado, sem gravar.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct PreviaDocumental {
    pub(crate) bloco: String,
    pub(crate) documento: String,
    /// `None`: o detalhe observado não é completo e não contém o documento.
    pub(crate) presente_antes: Option<bool>,
    pub(crate) requisicoes: usize,
    pub(crate) confirmado: bool,
    pub(crate) alterado: bool,
    pub(crate) presente_depois: Option<bool>,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroIncluir {
    Abertura(#[from] ErroDocumentoObter),
    Barra(#[from] ErroBlocoNaBarra),
    /// A barra do documento não oferece "Incluir em Bloco de Assinatura".
    AcaoNaoOferecidaNaBarra,
    Cookie(#[from] CookieIlegivel),
    Tela(#[from] FalhaDaTela),
    Html(#[from] ErroHtml),
    /// `selBloco` não oferece o bloco pedido.
    BlocoNaoOferecido,
    /// A tela de escolha não lista o documento pedido.
    DocumentoForaDaTela,
    LeituraDoBloco(#[from] FalhaLeitura),
    Transporte(#[from] FalhaTransporte),
    Formulario(#[from] crate::formulario_web::ErroSubmissaoFormulario),
    CharsetNaoSuportado,
    PertencimentoNaoObservado,
    NaoConfirmado,
}

impl ErroIncluir {
    pub(crate) fn mensagem(&self) -> String {
        match self {
            Self::Abertura(e) => e.mensagem().to_owned(),
            Self::AcaoNaoOferecidaNaBarra => {
                "a barra do documento não oferece incluir em bloco de assinatura; nada foi incluído"
                    .to_owned()
            }
            Self::BlocoNaoOferecido => {
                "o bloco pedido não está entre os blocos de assinatura oferecidos pela tela; nada foi incluído"
                    .to_owned()
            }
            Self::DocumentoForaDaTela => {
                "a tela de escolha de bloco não lista o documento pedido; nada foi incluído".to_owned()
            }
            Self::PertencimentoNaoObservado => {
                "não se comprovou que o documento esteja ausente do bloco; nenhuma escrita realizada".to_owned()
            }
            Self::NaoConfirmado => {
                "inclusão não confirmada; consulte o bloco antes de tentar novamente".to_owned()
            }
            Self::LeituraDoBloco(e) => e.mensagem().to_owned(),
            Self::Transporte(e) => e.mensagem().to_owned(),
            Self::Html(e) => e.mensagem.to_owned(),
            outro => format!("a inclusão em bloco não pôde ser preparada: {outro}; nada foi incluído"),
        }
    }
}

fn atributo(tag: &tl::HTMLTag<'_>, nome: &'static str) -> Result<Option<String>, ErroHtml> {
    let fora = || ErroHtml::valor("tela_de_escolher_bloco_fora_da_forma_observada");
    let lidos = atributos(tag).map_err(|_| fora())?;
    match lidos.valor(nome).map_err(|_| fora())? {
        Some(bruto) => decodificar_atributo(bruto).map(Some).ok_or_else(fora),
        None => Ok(None),
    }
}

/// Lê o `frmBlocoEscolher`: um formulário, um `selBloco`.
pub(crate) fn ler_tela_de_escolha(html: &str) -> Result<TelaDeEscolha, ErroHtml> {
    let fora = || ErroHtml::valor("tela_de_escolher_bloco_fora_da_forma_observada");
    let dom = tl::parse(html, tl::ParserOptions::default()).map_err(|_| fora())?;
    let parser = dom.parser();
    let tags = |seletor: &str| -> Vec<&tl::HTMLTag<'_>> {
        dom.query_selector(seletor)
            .into_iter()
            .flatten()
            .filter_map(|n| n.get(parser).and_then(tl::Node::as_tag))
            .collect()
    };
    let formularios = tags("form#frmBlocoEscolher");
    match formularios.len() {
        0 => return Err(ErroHtml::ausente("formulario_de_escolher_bloco_ausente")),
        1 => {}
        _ => {
            return Err(ErroHtml::repetido(
                "formularios_de_escolher_bloco_repetidos",
            ));
        }
    }
    let selecoes = tags("select#selBloco");
    let selecao = match selecoes.as_slice() {
        [] => return Err(ErroHtml::ausente("selecao_de_bloco_ausente")),
        [selecao] => *selecao,
        _ => return Err(ErroHtml::repetido("selecoes_de_bloco_repetidas")),
    };
    let mut blocos = Vec::new();
    for filho in selecao.children().all(parser) {
        let Some(opcao) = filho.as_tag().filter(|t| t.name() == "option") else {
            continue;
        };
        let valor = atributo(opcao, "value")?.ok_or_else(fora)?;
        if valor == "null" {
            continue;
        }
        if valor.is_empty() || !valor.bytes().all(|b| b.is_ascii_digit()) {
            return Err(fora());
        }
        blocos.push(valor);
    }
    let mut documentos = Vec::new();
    for caixa in tags("input") {
        let Some(id) = atributo(caixa, "id")? else {
            continue;
        };
        if !id.starts_with("chkDocumentosItem") {
            continue;
        }
        documentos.push(atributo(caixa, "title")?.ok_or_else(fora)?);
    }
    Ok(TelaDeEscolha { blocos, documentos })
}

/// Identidade pública e intenção do comando, sem transporte ou estado duplicado.
pub(crate) struct PedidoDaInclusao<'a> {
    pub(crate) bloco: &'a str,
    pub(crate) documento: &'a NumeroDocumento,
    pub(crate) confirmar: bool,
}

/// Prévia ou confirmação: formulário fresco, no máximo um POST, releitura.
pub(crate) fn prever_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    referencias: &ReferenciasDaSessao,
    unidade: Option<&str>,
    pedido: PedidoDaInclusao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<PreviaDocumental, ErroIncluir> {
    let PedidoDaInclusao { bloco, documento, confirmar } = pedido;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(ErroIncluir::Transporte(
        FalhaTransporte::OrigemNaoAutorizada,
    ))?;
    let mut rede = Contador::novo(envio);
    let pedido = PedidoDoDocumento {
        numero: documento,
        unidade,
        processo: None,
        anexo: PedidoDoAnexo::default(),
    };
    let aberto = abrir_documento_na_arvore(sessao, &pedido, indice, &mut rede)?;
    let link = ler_bloco_escolher_da_barra(
        &instalacao,
        &aberto.html,
        &aberto.id_documento,
        &aberto.processo.id,
    )?
    .ok_or(ErroIncluir::AcaoNaoOferecidaNaBarra)?;
    let cookie = ler_cookie(sessao)?;
    let (html, mime) = obter_tela(&mut rede, &cookie, link.as_str())?;
    let tela = ler_tela_de_escolha(&html)?;
    if !tela.blocos.iter().any(|b| b == bloco) {
        return Err(ErroIncluir::BlocoNaoOferecido);
    }
    if !tela.documentos.contains(&aberto.numero) {
        return Err(ErroIncluir::DocumentoForaDaTela);
    }
    let lido = ler_tipado_via(
        referencias,
        unidade,
        TipoBloco::Assinatura,
        Some(bloco),
        None,
        &mut rede,
    )?;
    let Leitura::Detalhe(detalhe) = lido.leitura else {
        unreachable!("com número, a leitura é o detalhe do bloco");
    };
    let presente = detalhe
        .itens
        .iter()
        .any(|item| item.documento.as_deref() == Some(aberto.numero.as_str()));
    let completo = detalhe.itens.len() == detalhe.total_declarado;
    let presente_antes = (presente || completo).then_some(presente);
    let mut alterado = false;
    let mut presente_depois = None;
    if confirmar && presente_antes == Some(false) {
        confirmar_documento(
            sessao,
            referencias,
            unidade,
            bloco,
            &aberto,
            TelaDaInclusao {
                url: link.as_str(),
                html: &html,
                mime: &mime,
            },
            &mut rede,
        )?;
        alterado = true;
        presente_depois = Some(true);
    } else if confirmar && presente_antes.is_none() {
        return Err(ErroIncluir::PertencimentoNaoObservado);
    }
    Ok(PreviaDocumental {
        bloco: bloco.to_owned(),
        documento: aberto.numero,
        presente_antes,
        confirmado: confirmar,
        alterado,
        presente_depois,
        requisicoes: usize::from(rede.feitas()),
    })
}


/// A tela fresca da oferta de inclusão, sem persistir HTML assinado.
struct TelaDaInclusao<'a> {
    url: &'a str,
    html: &'a str,
    mime: &'a str,
}

/// Envio causal único e uma leitura independente. Ambos são necessários
/// para declarar alteração, mesmo se a resposta 302 foi observada.
fn confirmar_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    referencias: &ReferenciasDaSessao,
    unidade: Option<&str>,
    bloco: &str,
    aberto: &DocumentoNaArvoreAberto,
    tela: TelaDaInclusao<'_>,
    rede: &mut Contador<'_, V>,
) -> Result<(), ErroIncluir> {
    let instalacao = Instalacao::interpretar(sessao.origem)
        .ok_or(FalhaTransporte::OrigemNaoAutorizada)?;
    let codificacao = codificacao_da_tela(tela.mime).ok_or(ErroIncluir::CharsetNaoSuportado)?;
    let servida = TelaServida {
        url: tela.url,
        html: tela.html,
        codificacao,
    };
    let submissao = derivar_submissao(
        &INCLUIR_DOCUMENTO,
        &instalacao,
        &servida,
        "",
        &[("selBloco", bloco), ("chkDocumentosItem0", &aberto.id_documento)],
    )?;
    // Submissão consumida, sem possibilidade de segundo POST.
    let resposta = enviar_submissao_via(submissao, sessao.cookie, |sub, cookie| {
        rede.post(sub, cookie)
    });
    let aceita = match resposta {
        Ok(RespostaDoEnvio::Redirecionamento(r)) => redirecionamento_da_inclusao(
            r.localizacao(),
            r.url_do_envio(),
            &aberto.processo.id,
            &aberto.id_documento,
        ),
        _ => false,
    };
    let relido = ler_tipado_via(
        referencias, unidade, TipoBloco::Assinatura, Some(bloco), None, rede,
    );
    let conferido = matches!(
        relido,
        Ok(crate::bloco_cli::Lido { leitura: Leitura::Detalhe(ref detalhe), .. })
            if detalhe.itens.iter().any(|item| {
                item.documento.as_deref() == Some(aberto.numero.as_str())
            })
    );
    if !aceita || !conferido {
        return Err(ErroIncluir::NaoConfirmado);
    }
    Ok(())
}

const INCLUIR_DOCUMENTO: OperacaoDeFormulario = OperacaoDeFormulario {
    acao: "bloco_escolher",
    formulario: "frmBlocoEscolher",
    envio: EnvioDoFormulario::BotaoDeEnvio("sbmIncluir"),
    campos_ocultos_exigidos: &["hdnDocumentosItens", "hdnDocumentosItensSelecionados"],
    campos_ocultos_reescritos: &[],
    campos_usuario: &["selBloco", "chkDocumentosItem0"],
    resultado: ResultadoDoEnvio::ConferidoPorLeituraPosterior,
    unidade: ConferenciaDaUnidade::NaTelaDeOrigemDoLink,
    efeito: EfeitoAvaliado::AlteraEstadoDoProcesso {
        o_que: EstadoDoProcesso::DocumentoIncluidoNoBloco,
    },
};

/// O 302 é apenas aceitação causal da inclusão quando vinculado à mesma
/// identidade e à ação publicada, sem seguir redirecionamento.
fn redirecionamento_da_inclusao(
    location: Option<&str>,
    url_do_envio: &str,
    id_procedimento: &str,
    id_documento: &str,
) -> bool {
    let Some(consulta) = location.and_then(|loc| consulta_do_controlador(loc, url_do_envio)) else {
        return false;
    };
    let valores = |campo: &str| -> Vec<&str> {
        consulta
            .split('&')
            .filter_map(|item| item.split_once('='))
            .filter(|(nome, _)| *nome == campo)
            .map(|(_, valor)| valor)
            .collect()
    };
    valores("acao") == ["bloco_escolher"]
        && valores("id_procedimento") == [id_procedimento]
        && valores("id_documento") == [id_documento]
}

#[cfg(test)]
#[path = "bloco_incluir/tests.rs"]
mod tests;
