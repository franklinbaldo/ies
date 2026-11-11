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
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;

/// URL absoluta do conteúdo, assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDoConteudo(String);

impl EnderecoDoConteudo {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }

    /// Endereço assinado oferecido no cadastro, nunca construído pelo IES.
    pub(crate) fn do_cadastro(
        inst: &Instalacao,
        href: &str,
        id: &str,
    ) -> Result<Self, ErroConteudoNaArvore> {
        if acao_unica(href) != Some(AcaoDoSei::DocumentoDownloadAnexo)
            || id_anexo_unico(href) != Some(id)
        {
            return Err(ErroConteudoNaArvore::LinkAnexoCadastroInvalido);
        }
        conferir_endereco_relativo_por(
            inst,
            href,
            AcaoDoSei::DocumentoDownloadAnexo,
            "id_anexo",
            id,
        )
        .map(Self)
        .map_err(ErroConteudoNaArvore::LinkAnexoCadastroRecusado)
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroConteudoNaArvore {
    LinkAnexoCadastroRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    EnderecoDoConteudoRecusado(#[source] crate::arvore_processo::endereco::EnderecoRecusado),
    EnderecoDoConteudoDeOutroDocumento,
    DocumentoAusenteDaArvore,
    EnderecoDoConteudoNaoReconhecido,
    LinkAnexoCadastroInvalido,
    LinkDoConteudoAusenteNaArvore,
    LinkDoConteudoForaDaFormaObservada,
    LinkDoConteudoRepetidoNaArvore,
}

/// Lê o `.src` do `DOCUMENTO` de id `id_documento` numa árvore já
/// conferida por [`super::ler_arvore`].
pub(crate) fn ler_conteudo_na_arvore(
    instalacao: &Instalacao,
    html: &str,
    id_documento: &str,
) -> Result<ConteudoNaArvore, ErroConteudoNaArvore> {
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
        .ok_or(ErroConteudoNaArvore::DocumentoAusenteDaArvore)?;
    let mut lidos = Vec::new();
    for linha in html.split('\n') {
        match ler_src(linha) {
            Ok(Some((i, texto))) if i == indice => lidos.push(texto),
            Err(_) if indice_do_src(linha) == Some(indice) => {
                return Err(ErroConteudoNaArvore::LinkDoConteudoForaDaFormaObservada);
            }
            _ => {}
        }
    }
    let src = match lidos.as_slice() {
        [] => return Err(ErroConteudoNaArvore::LinkDoConteudoAusenteNaArvore),
        [um] => um,
        _ => return Err(ErroConteudoNaArvore::LinkDoConteudoRepetidoNaArvore),
    };
    if src.is_empty() {
        return Ok(ConteudoNaArvore::Vazio);
    }
    match acao_unica(src) {
        Some(AcaoDoSei::DocumentoDownloadAnexo) => {
            // O link não traz `id_documento`: o id conferido é o `id_anexo`
            // único e decimal do próprio link; a ligação é o nó (Refs #60).
            let id_anexo = id_anexo_unico(src)
                .ok_or(ErroConteudoNaArvore::EnderecoDoConteudoNaoReconhecido)?;
            conferir_endereco_relativo_por(
                instalacao,
                src,
                AcaoDoSei::DocumentoDownloadAnexo,
                "id_anexo",
                id_anexo,
            )
            .map(|url| ConteudoNaArvore::DownloadDeAnexo(EnderecoDoConteudo(url)))
            .map_err(ErroConteudoNaArvore::EnderecoDoConteudoRecusado)
        }
        Some(AcaoDoSei::DocumentoVisualizar) => conferir_endereco_relativo_por(
            instalacao,
            src,
            AcaoDoSei::DocumentoVisualizar,
            "id_documento",
            id_documento,
        )
        .map(|url| ConteudoNaArvore::Visualizar(EnderecoDoConteudo(url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => {
                ErroConteudoNaArvore::EnderecoDoConteudoNaoReconhecido
            }
            EnderecoRecusado::DeOutroProcesso => {
                ErroConteudoNaArvore::EnderecoDoConteudoDeOutroDocumento
            }
        }),
        _ => Err(ErroConteudoNaArvore::EnderecoDoConteudoNaoReconhecido),
    }
}

/// O `id_anexo` único e decimal de `controlador.php?…`.
fn id_anexo_unico(src: &str) -> Option<&str> {
    let consulta = src.strip_prefix("controlador.php?")?;
    let id = ConsultaDoControlador::ler(consulta)
        .ok()?
        .unico("id_anexo")
        .ok()?;
    (!id.is_empty() && id.bytes().all(|b| b.is_ascii_digit())).then_some(id)
}

/// A `acao` única de `controlador.php?…`, sem conferir o resto.
fn acao_unica(src: &str) -> Option<AcaoDoSei> {
    let consulta = src.strip_prefix("controlador.php?")?;
    ConsultaDoControlador::ler(consulta).ok()?.acao().ok()
}
