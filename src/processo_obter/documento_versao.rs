//! Versão N e cópia de trabalho de `ies documento DOC` (Refs #88;
//! contracts/cli/documento-obter-versoes.md), depois da árvore conferida:
//! a barra do `DOCUMENTO` publica a lista de versões (link assinado) e a
//! edição; a lista (`documento_versao_listar`) publica, por linha, o
//! `documento_visualizar` assinado da versão. Nenhum link montado, nenhum
//! editor. `versao_atual` só quando a lista foi lida (decisão de custo).
use super::super::EnvioDoProcesso;
use super::super::abertura::Rede;
use super::{E, ler_conteudo};
use crate::arvore_processo::EnderecoDoConteudo;
use crate::documento_versoes::VersaoDocumento;
use crate::instalacao::Instalacao;

/// O que a leitura pede além do conteúdo atual.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PedidoDeVersoes {
    /// `--versao N`: lê a versão N em vez do conteúdo atual.
    pub(crate) versao: Option<VersaoDocumento>,
    /// `--saida` sem `--sem-copia`: preparar `DOC.vN.html` quando seguro.
    pub(crate) copia: bool,
}

/// Por que `versao_atual` não foi publicada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemVersaoAtual {
    NaoPedida,
    ListaNaoPublicadaNaArvore,
    /// A cópia não era possível; a lista (1 GET) não foi pedida.
    CopiaNaoPreparavel,
}

impl SemVersaoAtual {
    pub(crate) fn motivo(self) -> &'static str {
        match self {
            Self::NaoPedida => "nao_pedida",
            Self::ListaNaoPublicadaNaArvore => "lista_de_versoes_nao_publicada_na_arvore",
            Self::CopiaNaoPreparavel => "copia_nao_preparavel",
        }
    }
}

/// Por que a cópia de trabalho não foi preparada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SemCopia {
    NaoPedida,
    ListaNaoPublicadaNaArvore,
    DocumentoAssinado,
    EdicaoNaoPublicadaNaArvore,
    VersaoHistorica,
    ConteudoComLinkAssinado,
}

impl SemCopia {
    pub(crate) fn motivo(self) -> &'static str {
        match self {
            Self::NaoPedida => "nao_pedida",
            Self::ListaNaoPublicadaNaArvore => "lista_de_versoes_nao_publicada_na_arvore",
            Self::DocumentoAssinado => "documento_assinado",
            Self::EdicaoNaoPublicadaNaArvore => "edicao_nao_publicada_na_arvore",
            Self::VersaoHistorica => "versao_historica",
            Self::ConteudoComLinkAssinado => "conteudo_com_link_assinado",
        }
    }
}

/// O HTML servido da versão atual, que a CLI grava como `DOC.vN.html`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct CopiaDeTrabalho {
    pub(crate) versao: VersaoDocumento,
    pub(crate) html: String,
}

impl std::fmt::Debug for CopiaDeTrabalho {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CopiaDeTrabalho(v{}, <html>)", self.versao.numero())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct VersoesDoDocumento {
    /// N lido com `--versao N`.
    pub(crate) lida: Option<VersaoDocumento>,
    pub(crate) atual: Result<VersaoDocumento, SemVersaoAtual>,
    pub(crate) copia: Result<CopiaDeTrabalho, SemCopia>,
}

impl VersoesDoDocumento {
    fn nao_pedidas() -> Self {
        Self {
            lida: None,
            atual: Err(SemVersaoAtual::NaoPedida),
            copia: Err(SemCopia::NaoPedida),
        }
    }
}

/// O documento (já conferido na árvore) e onde está o conteúdo atual.
pub(super) struct NoConferido<'a> {
    pub(super) instalacao: &'a Instalacao,
    pub(super) arvore: &'a str,
    pub(super) id_documento: &'a str,
    pub(super) id_procedimento: &'a str,
    pub(super) numero: &'a str,
    pub(super) conteudo: &'a EnderecoDoConteudo,
}

/// Tipo, texto e versões do documento, com o mínimo de GETs do pedido.
pub(super) fn ler_conteudo_e_versoes<V: EnvioDoProcesso>(
    rede: &mut Rede<'_, V>,
    no: &NoConferido<'_>,
    pedido: PedidoDeVersoes,
) -> Result<(String, String, VersoesDoDocumento), E> {
    // RED: shim que ignora o pedido.
    let _ = (
        pedido,
        no.instalacao,
        no.arvore,
        no.id_documento,
        no.id_procedimento,
    );
    let (tipo, texto) = ler_conteudo(rede, no.conteudo, no.numero)?;
    Ok((tipo, texto, VersoesDoDocumento::nao_pedidas()))
}
