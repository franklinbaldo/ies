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
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo_concluir::escrita::{
    Contador, CookieIlegivel, FalhaDaTela, ler_cookie, obter_tela,
};
use crate::processo_obter::{
    EnvioDoProcesso, ErroDocumentoObter, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    abrir_documento_na_arvore,
};
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::ReferenciasDaSessao;

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
}

impl ErroIncluir {}

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

/// Prévia: abertura do documento, tela de escolha e detalhe do bloco; sem POST.
pub(crate) fn prever_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    referencias: &ReferenciasDaSessao,
    unidade: Option<&str>,
    bloco: &str,
    documento: &NumeroDocumento,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<PreviaDocumental, ErroIncluir> {
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
    let (html, _) = obter_tela(&mut rede, &cookie, link.as_str())?;
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
    Ok(PreviaDocumental {
        bloco: bloco.to_owned(),
        documento: aberto.numero,
        presente_antes: (presente || completo).then_some(presente),
        requisicoes: usize::from(rede.feitas()),
    })
}

#[cfg(test)]
#[path = "bloco_incluir/tests.rs"]
mod tests;
