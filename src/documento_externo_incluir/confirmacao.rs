//! #332 — upload temporário, cadastro definitivo e releitura independente.
//! Testes técnicos auxiliares aos cenários BDD-332-03 e BDD-332-04.
use super::{
    cadastro::CadastroExterno,
    cadastro_envio::{FalhaCadastroEnvio, preparar_cadastro_definitivo},
    recibo::{FalhaUpload, ReciboUpload},
};
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore};
use crate::documento_criar::ErroConferenciaDeCriacao;
use crate::formulario_web::{
    ErroSubmissaoFormulario, RespostaDoEnvio, SubmissaoDerivada, TelaServida,
};
use crate::instalacao::Instalacao;
use crate::processo_obter::ErroProcessoObter;

pub(crate) struct FotoProcesso {
    pub(crate) id: String,
    pub(crate) documentos: Vec<DocumentoNaArvore>,
    pub(crate) pastas_nao_carregadas: bool,
}
impl FotoProcesso {
    pub(crate) fn da_arvore(arvore: &ArvoreLida) -> Self {
        Self {
            id: arvore.processo.id.clone(),
            documentos: arvore.documentos.clone(),
            pastas_nao_carregadas: arvore.pastas_nao_carregadas,
        }
    }
}
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaConfirmacao {
    FotoInicialIncompleta,
    FotoInicialInconsistente,
    UploadRecusado(FalhaUpload),
    UploadIncerto(FalhaUpload),
    TemporarioSemCadastro(FalhaCadastroEnvio),
    CadastroIncerto(#[source] ErroSubmissaoFormulario),
    ReleituraIncerta(#[source] ErroProcessoObter),
    /// A árvore lida depois do cadastro não prova o documento criado.
    CriacaoNaoConferida(#[source] ErroConferenciaDeCriacao),
    NaoConfirmado,
}
#[cfg(test)]
impl FalhaConfirmacao {
    pub(crate) fn temporario_possivel(&self) -> bool {
        !matches!(
            self,
            Self::FotoInicialIncompleta | Self::FotoInicialInconsistente | Self::UploadRecusado(_)
        )
    }
}
/// Upload e cadastro são escritos no máximo uma vez cada. A releitura é
/// independente mesmo quando o POST definitivo retorna falha ou HTML genérico.
/// Essa função não habilita a confirmação na CLI; ainda falta prova live.
#[allow(clippy::too_many_arguments)]
pub(crate) fn confirmar_sequencia<T, U, C, L>(
    transporte: &mut T,
    instalacao: &Instalacao,
    tela: &TelaServida<'_>,
    unidade: &str,
    cadastro: &CadastroExterno,
    antes: &FotoProcesso,
    tipo: &str,
    enviar_upload: U,
    enviar_cadastro: C,
    reler: L,
) -> Result<String, FalhaConfirmacao>
where
    U: FnOnce(&mut T) -> Result<ReciboUpload, FalhaUpload>,
    C: FnOnce(&mut T, SubmissaoDerivada) -> Result<RespostaDoEnvio, ErroSubmissaoFormulario>,
    L: FnOnce(&mut T) -> Result<FotoProcesso, ErroProcessoObter>,
{
    if antes.pastas_nao_carregadas {
        return Err(FalhaConfirmacao::FotoInicialIncompleta);
    }
    let mut ids = std::collections::HashSet::new();
    if antes.id.is_empty()
        || antes
            .documentos
            .iter()
            .any(|doc| doc.id.is_empty() || !ids.insert(doc.id.as_str()))
    {
        return Err(FalhaConfirmacao::FotoInicialInconsistente);
    }

    let recibo = enviar_upload(transporte).map_err(|erro| match erro {
        FalhaUpload::Recusado { .. } | FalhaUpload::PreparacaoMultipart(_) => {
            FalhaConfirmacao::UploadRecusado(erro)
        }
        FalhaUpload::NaoConfirmado { .. }
        | FalhaUpload::EnvioSemResposta(_)
        | FalhaUpload::CorpoNaoLido(_)
        | FalhaUpload::CorpoIlegivel(_)
        | FalhaUpload::TamanhoDoReciboIlegivel(_) => FalhaConfirmacao::UploadIncerto(erro),
    })?;
    let submissao = preparar_cadastro_definitivo(instalacao, tela, unidade, cadastro, &recibo)
        .map_err(FalhaConfirmacao::TemporarioSemCadastro)?;
    // Mesmo após um erro de envio a releitura pode mostrar o efeito: nunca
    // transformar uma falha de resposta em permissão para repetir o POST.
    let envio = enviar_cadastro(transporte, submissao);
    let leitura = reler(transporte);
    let resposta = envio.map_err(FalhaConfirmacao::CadastroIncerto)?;
    let depois = leitura.map_err(FalhaConfirmacao::ReleituraIncerta)?;

    if depois.id != antes.id || depois.pastas_nao_carregadas {
        return Err(FalhaConfirmacao::NaoConfirmado);
    }
    // A árvore rotula o externo com o número entre parênteses (UnB, live
    // 09/11/2026 e cassette documento-mover-cancelar-unb): `Anexo (0012272)`.
    let numero = crate::documento_criar::conferir_criacao_com_rotulo(
        &antes.documentos,
        &depois.documentos,
        |rotulo, numero| rotulo == format!("{tipo} ({numero})"),
    )
    .map_err(FalhaConfirmacao::CriacaoNaoConferida)?;
    let novo = depois
        .documentos
        .iter()
        .find(|doc| doc.numero_sei.as_deref() == Some(numero.as_str()))
        .ok_or(FalhaConfirmacao::NaoConfirmado)?;
    // A árvore publica "documento_pdf" para PDFs externos (UnB/GO).
    // Número, rótulo e redirect não bastam para confirmar a inclusão externa:
    // documento interno concorrente ou natureza não observada são incertos.
    if novo.natureza.icone != "documento_pdf" || novo.natureza.cancelamento.cancelado() {
        return Err(FalhaConfirmacao::NaoConfirmado);
    }
    let RespostaDoEnvio::Redirecionamento(redir) = resposta else {
        return Err(FalhaConfirmacao::NaoConfirmado);
    };
    if !redir.localizacao().is_some_and(|location| {
        crate::documento_criar::redirecionamento_do_mesmo_controlador(
            location,
            redir.url_do_envio(),
            &novo.id,
        )
    }) {
        return Err(FalhaConfirmacao::NaoConfirmado);
    }
    Ok(numero)
}

#[cfg(test)]
#[path = "tests_confirmacao.rs"]
mod tests;
