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
    /// `documento_download_anexo` (documento externo): não é seguido.
    DownloadDeAnexo,
    /// `''`: a árvore não publica conteúdo (sem acesso ou cancelado).
    Vazio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroConteudoNaArvore {
    DocumentoAusenteDaArvore,
    SrcDoDocumentoAusente,
    SrcDoDocumentoRepetido,
    SrcDoDocumentoForaDaFormaObservada,
    EnderecoDoConteudoNaoReconhecido,
    EnderecoDoConteudoDeOutroDocumento,
}

impl ErroConteudoNaArvore {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::DocumentoAusenteDaArvore => "documento_ausente_da_arvore",
            Self::SrcDoDocumentoAusente => "link_do_conteudo_ausente_na_arvore",
            Self::SrcDoDocumentoRepetido => "link_do_conteudo_repetido_na_arvore",
            Self::SrcDoDocumentoForaDaFormaObservada => "link_do_conteudo_fora_da_forma_observada",
            Self::EnderecoDoConteudoNaoReconhecido => "endereco_do_conteudo_nao_reconhecido",
            Self::EnderecoDoConteudoDeOutroDocumento => "endereco_do_conteudo_de_outro_documento",
        }
    }
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
        .ok_or(E::DocumentoAusenteDaArvore)?;
    let mut lidos = Vec::new();
    for linha in html.split('\n') {
        match ler_src(linha) {
            Ok(Some((i, texto))) if i == indice => lidos.push(texto),
            Err(_) if indice_do_src(linha) == Some(indice) => {
                return Err(E::SrcDoDocumentoForaDaFormaObservada);
            }
            _ => {}
        }
    }
    let src = match lidos.as_slice() {
        [] => return Err(E::SrcDoDocumentoAusente),
        [um] => um,
        _ => return Err(E::SrcDoDocumentoRepetido),
    };
    if src.is_empty() {
        return Ok(ConteudoNaArvore::Vazio);
    }
    match acao_unica(src) {
        Some(DOWNLOAD) => Ok(ConteudoNaArvore::DownloadDeAnexo),
        Some(VISUALIZAR) => conferir_endereco_relativo_por(
            instalacao,
            src,
            VISUALIZAR,
            "id_documento",
            id_documento,
        )
        .map(|url| ConteudoNaArvore::Visualizar(EnderecoDoConteudo(url)))
        .map_err(|recusa| match recusa {
            EnderecoRecusado::NaoReconhecido => E::EnderecoDoConteudoNaoReconhecido,
            EnderecoRecusado::DeOutroProcesso => E::EnderecoDoConteudoDeOutroDocumento,
        }),
        _ => Err(E::EnderecoDoConteudoNaoReconhecido),
    }
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
