//! O link do conteúdo de um `DOCUMENTO` (Refs #60,
//! contracts/cli/documento-obter-web.md): a linha `Nos[i].src = '…';` do
//! nó cujo id (a1) é o pedido. Forma observada nas sete instalações do
//! acervo público: documento interno (e e-mail) com
//! `controlador.php?acao=documento_visualizar&…&id_documento=<id>&…&infra_hash=…`;
//! externo (PDF, TXT, HTML recebido) com `acao=documento_download_anexo`
//! e `id_anexo`; sem acesso ou cancelado, `''`. O link é **assinado e
//! publicado pela árvore**: o IES nunca monta um (ADR 0003).
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo_por};
use super::literais::{Literal, indice_do_src, ler_declaracao, ler_src};
use crate::erro_html::ErroHtml;
use crate::instalacao::Instalacao;

const VISUALIZAR: &str = "documento_visualizar";
const DOWNLOAD: &str = "documento_download_anexo";

/// URL absoluta do conteúdo, assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDoConteudo(String);

impl EnderecoDoConteudo {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Endereço assinado oferecido no cadastro, nunca construído pelo IES.
    pub(crate) fn do_cadastro(inst: &Instalacao, href: &str, id: &str) -> Result<Self, ErroHtml> {
        if acao_unica(href) != Some(DOWNLOAD) || id_anexo_unico(href) != Some(id) {
            return Err(ErroHtml::valor("link_anexo_cadastro_invalido"));
        }
        conferir_endereco_relativo_por(inst, href, DOWNLOAD, "id_anexo", id)
            .map(Self)
            .map_err(|_| ErroHtml::valor("link_anexo_cadastro_invalido"))
    }
}

impl std::fmt::Debug for EnderecoDoConteudo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDoConteudo(<redigido>)")
    }
}

/// O que o `.src` do nó publica.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConteudoNaArvore {
    /// `documento_visualizar` do mesmo `id_documento`, conferido.
    Visualizar(EnderecoDoConteudo),
    /// `documento_download_anexo` (documento externo) com um único
    /// `id_anexo` decimal, conferido (Refs #60, fatia 3).
    DownloadDeAnexo(EnderecoDoConteudo),
    /// `''`: a árvore não publica conteúdo (sem acesso ou cancelado).
    Vazio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroConteudoNaArvore {
    EnderecoDoConteudoDeOutroDocumento,
    #[causa(transparente)]
    Html(ErroHtml),
}

/// Lê o `.src` do `DOCUMENTO` de id `id_documento` numa árvore já
/// conferida por [`super::ler_arvore`].
pub(crate) fn ler_conteudo_na_arvore(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
) -> Result<ConteudoNaArvore, ErroConteudoNaArvore> {
    use ErroConteudoNaArvore as E;
    let indice = html
        .split('\n')
        .find_map(|linha| {
            let (i, argumentos) = ler_declaracao(linha).ok()??;
            match argumentos.as_slice() {
                [Literal::Texto(tipo), Literal::Texto(id), ..]
                    if tipo == "DOCUMENTO" && id == id_documento =>
                {
                    Some(i)
                }
                _ => None,
            }
        })
        .ok_or(ErroHtml::ausente("documento_ausente_da_arvore"))?;
    let mut lidos = Vec::new();
    for linha in html.split('\n') {
        match ler_src(linha) {
            Ok(Some((i, texto))) if i == indice => lidos.push(texto),
            Err(_) if indice_do_src(linha) == Some(indice) => {
                return Err(ErroHtml::valor("link_do_conteudo_fora_da_forma_observada").into());
            }
            _ => {}
        }
    }
    let src = match lidos.as_slice() {
        [] => return Err(ErroHtml::ausente("link_do_conteudo_ausente_na_arvore").into()),
        [um] => um,
        _ => return Err(ErroHtml::repetido("link_do_conteudo_repetido_na_arvore").into()),
    };
    if src.is_empty() {
        return Ok(ConteudoNaArvore::Vazio);
    }
    match acao_unica(src) {
        Some(DOWNLOAD) => {
            // O link não traz `id_documento`: o id conferido é o `id_anexo`
            // único e decimal do próprio link; a ligação é o nó (Refs #60).
            let id_anexo = id_anexo_unico(src)
                .ok_or(ErroHtml::valor("endereco_do_conteudo_nao_reconhecido"))?;
            conferir_endereco_relativo_por(instalacao, src, DOWNLOAD, "id_anexo", id_anexo)
                .map(|url| ConteudoNaArvore::DownloadDeAnexo(EnderecoDoConteudo(url)))
                .map_err(|_| E::Html(ErroHtml::valor("endereco_do_conteudo_nao_reconhecido")))
        }
        Some(VISUALIZAR) => conferir_endereco_relativo_por(
            instalacao,
            src,
            VISUALIZAR,
            "id_documento",
            id_documento,
        )
        .map(|url| ConteudoNaArvore::Visualizar(EnderecoDoConteudo(url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                E::Html(ErroHtml::valor("endereco_do_conteudo_nao_reconhecido"))
            }
            EnderecoRecusado::DeOutroProcesso => E::EnderecoDoConteudoDeOutroDocumento,
        }),
        _ => Err(ErroHtml::valor("endereco_do_conteudo_nao_reconhecido").into()),
    }
}

/// O `id_anexo` único e decimal de `controlador.php?…`.
fn id_anexo_unico(src: &str) -> Option<&str> {
    let consulta = src.strip_prefix("controlador.php?")?;
    let mut ids = consulta
        .split('&')
        .filter_map(|par| par.strip_prefix("id_anexo="));
    let id = ids.next()?;
    (ids.next().is_none() && !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

/// A `acao` única de `controlador.php?…`, sem conferir o resto.
fn acao_unica(src: &str) -> Option<&str> {
    let consulta = src.strip_prefix("controlador.php?")?;
    let mut acoes = consulta
        .split('&')
        .filter_map(|par| par.strip_prefix("acao="));
    let acao = acoes.next()?;
    acoes.next().is_none().then_some(acao)
}
