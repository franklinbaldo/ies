//! `ies documento assinar DOC [--cargo TEXTO] [--confirmar]`
//! (contracts/cli/documento-assinar.md, Refs #174): abre o documento pelo
//! caminho de `ies documento DOC` até a árvore, lê a ação da barra e a
//! caneta, abre a tela do documento e a "Assinatura de Documento" (GETs sem
//! efeito observado) e confere o cargo; prévia sem POST; com `--confirmar`,
//! um POST com a senha do arquivo privado e uma leitura nova da árvore para
//! verificar. Nunca repete a escrita.
mod erro;
mod fluxo;
mod tela;

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod tests_apoio;
#[cfg(test)]
mod tests_fluxo;
#[cfg(test)]
mod tests_tela;

pub(crate) use erro::{ErroAssinar, FalhaDoEnvio};
pub(crate) use fluxo::assinar_documento;

use crate::arvore_processo::{Assinante, EdicaoNaBarra, ProcessoNaArvore};
use crate::documento::NumeroDocumento;
use crate::processo::NumeroProcesso;
use crate::sessao_login::Credenciais;

/// O que `--confirmar` exige: o cargo por extenso e a senha já lida.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Confirmacao<'a> {
    pub(crate) cargo: &'a str,
    pub(crate) credenciais: &'a Credenciais,
}

/// O número SEI, a `--unidade`, o `--processo`, o `--cargo` da prévia e,
/// com `--confirmar`, a confirmação.
pub(crate) struct PedidoDaAssinatura<'a> {
    pub(crate) numero: &'a NumeroDocumento,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) processo: Option<&'a NumeroProcesso>,
    pub(crate) cargo: Option<&'a str>,
    pub(crate) confirmacao: Option<Confirmacao<'a>>,
}

/// Sucesso: prévia, nada a fazer (já assinado pela sessão), ou assinado e
/// conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoAssinado {
    pub(crate) numero: String,
    pub(crate) id_documento: String,
    pub(crate) rotulo: String,
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) unidade: Option<String>,
    /// `txtUsuario` da tela.
    pub(crate) assinante: String,
    /// O cargo pedido (`--cargo`), já conferido entre as opções.
    pub(crate) cargo: Option<String>,
    pub(crate) cargo_da_tela: Option<String>,
    pub(crate) cargos: Vec<String>,
    /// A caneta antes.
    pub(crate) assinaturas: Vec<Assinante>,
    pub(crate) edicao: EdicaoNaBarra,
    pub(crate) resultado: ResultadoDaAssinatura,
    pub(crate) requisicoes: u8,
}

/// As três decisões mutuamente exclusivas do fluxo de assinatura.
/// Dados de releitura e reconciliação só existem após uma assinatura
/// comprovada; a prévia e o caso idempotente não os podem carregar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ResultadoDaAssinatura {
    Previa,
    JaAssinadoPelaSessao {
        confirmacao_pedida: bool,
    },
    Comprovada {
        assinatura_depois: Assinante,
        edicao_depois: EdicaoNaBarra,
        reconciliado_por: Option<FalhaDoEnvio>,
    },
}

/// `edicao` como texto público.
pub(crate) fn edicao_como_texto(edicao: EdicaoNaBarra) -> &'static str {
    match edicao {
        EdicaoNaBarra::NaoAssinado => "nao_assinado",
        EdicaoNaBarra::Assinado => "assinado",
        EdicaoNaBarra::NaoPublicada => "nao_publicada",
        EdicaoNaBarra::ForaDaForma => "fora_da_forma",
    }
}
