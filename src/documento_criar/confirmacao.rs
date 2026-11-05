//! Fronteira de escrita única e releitura para documento criar (#246).
//! O transporte é injetável; a confirmação nunca repete a submissão.
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore};
use crate::documento_criar::conferir_criacao;
use crate::formulario_web::{RespostaDoEnvio, enviar_submissao_via};
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
    let resposta = enviar_submissao_via(submissao, cookie, |post, cookie| {
        enviar(transporte, post, cookie)
    });
    let depois = reler(transporte).map_err(|()| ErroPreviaCriacao::EfeitoIncerto)?;
    if antes.pastas_nao_carregadas
        || depois.pastas_nao_carregadas
        || antes.id != depois.id
    {
        return Err(ErroPreviaCriacao::EfeitoIncerto);
    }
    let numero = conferir_criacao(&antes.documentos, &depois.documentos, tipo)
        .map_err(|_| ErroPreviaCriacao::EfeitoIncerto)?;
    if !matches!(
        resposta,
        Ok(RespostaDoEnvio::Html(_) | RespostaDoEnvio::Redirecionamento(_))
    ) {
        return Err(ErroPreviaCriacao::EfeitoIncerto);
    }
    let _ = numero;
    // RED: o ramo de sucesso ainda recusa; o teste deve detectar a regressão.
    Err(ErroPreviaCriacao::EfeitoIncerto)
}
