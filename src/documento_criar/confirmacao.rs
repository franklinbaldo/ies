//! Fronteira de escrita única e releitura para documento criar (#246).
//! O transporte é injetável; a confirmação nunca repete a submissão.
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore};
use crate::documento_criar::fluxo::ErroPreviaCriacao;
use crate::formulario_web::{RespostaDoPost, SubmissaoDerivada};
use crate::transporte::FalhaTransporte;
use crate::transporte::sessao::CookieDaSessao;
use std::io::Read;

pub(super) struct FotoDaCriacao {
    pub(super) id: String,
    pub(super) documentos: Vec<DocumentoNaArvore>,
    pub(super) pastas_nao_carregadas: bool,
}
impl FotoDaCriacao {
    pub(super) fn da_arvore(arvore: &ArvoreLida) -> Self {
        Self {
            id: arvore.processo.id.clone(),
            documentos: arvore.documentos.clone(),
            pastas_nao_carregadas: arvore.pastas_nao_carregadas,
        }
    }
}

pub(super) fn postar_e_verificar<T, R, P, L>(
    transporte: &mut T,
    submissao: SubmissaoDerivada,
    cookie: &CookieDaSessao,
    antes: &FotoDaCriacao,
    tipo: &str,
    enviar: P,
    reler: L,
) -> Result<String, ErroPreviaCriacao>
where
    R: Read,
    P: FnOnce(&mut T, SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
    L: FnOnce(&mut T) -> Result<FotoDaCriacao, ()>,
{
    let _ = (transporte, submissao, cookie, antes, tipo, enviar, reler);
    Err(ErroPreviaCriacao::EfeitoIncerto)
}
