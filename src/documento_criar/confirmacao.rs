//! Fronteira de escrita única e releitura para documento criar (#246).
//! O transporte é injetável; a confirmação nunca repete a submissão.
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore};
use crate::documento_criar::{ErroCriar, conferir_criacao};
use crate::formulario_web::{ErroSubmissaoFormulario, RespostaDoPost, SubmissaoDerivada};
use crate::formulario_web::{RespostaDoEnvio, enviar_submissao_via};
use crate::processo_obter::ErroProcessoObter;
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

/// O 302 não é seguido, mas só serve de indício se seu destino
/// continuar no controlador HTTPS da própria instalação. Um Location
/// externo/inválido pode coincidir com criação concorrente na árvore.
pub(crate) fn redirecionamento_do_mesmo_controlador(
    localizacao: &str,
    url_do_envio: &str,
    id_novo: &str,
) -> bool {
    let Some(origem) = crate::url_observada::UrlHttps::interpretar(url_do_envio) else {
        return false;
    };
    if !origem.caminho.ends_with("/controlador.php")
        || !localizacao.bytes().all(|byte| (33..=126).contains(&byte))
        || localizacao.contains(['#', '\\'])
    {
        return false;
    }
    let consulta = if localizacao.starts_with("https://") {
        let Some(destino) = crate::url_observada::UrlHttps::interpretar(localizacao) else {
            return false;
        };
        if destino.host != origem.host || destino.caminho != origem.caminho {
            return false;
        }
        destino.consulta
    } else {
        // O SEI publica Location absoluto ou relativo ao controlador.
        // Nunca resolver caminhos arbitrários, //host nem outro esquema.
        localizacao
            .strip_prefix("controlador.php?")
            .or_else(|| localizacao.strip_prefix('?'))
            .or_else(|| localizacao.strip_prefix(&format!("/{}?", origem.caminho)))
    };
    consulta.is_some_and(|consulta| {
        // O PDF externo pode redirecionar à árvore do processo selecionando
        // explicitamente o documento, sem perder a exigência de identidade.
        // A resposta precisa identificar o próprio documento observado na
        // árvore, não apenas navegar para qualquer ação do controlador.
        let mut acoes = consulta
            .split('&')
            .filter_map(|campo| campo.strip_prefix("acao="));
        let mut ids = consulta
            .split('&')
            .filter_map(|campo| campo.strip_prefix("id_documento="));
        matches!(
            acoes.next().and_then(AcaoDoSei::ler),
            Some(
                AcaoDoSei::DocumentoVisualizar
                    | AcaoDoSei::ArvoreVisualizar
                    | AcaoDoSei::ProcedimentoVisualizar
            )
        ) && acoes.next().is_none()
            && ids.next() == Some(id_novo)
            && ids.next().is_none()
    })
}

/// O único POST de criação e a releitura que o comprova.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroConfirmacaoDaCriacao {
    /// Pasta fechada na foto inicial: nenhuma escrita é enviada.
    ArvoreIncompleta,
    /// IDs de documento repetidos/vazios ou processo sem ID antes do POST.
    FotoInicialInconsistente,
    /// A releitura não demonstra um documento novo único do tipo pedido.
    NaoConfirmado,
    /// A releitura não traz um documento novo do tipo pedido; guarda o motivo.
    NaoDemonstradoNaArvore(#[source] ErroCriar),
    /// O POST pode ter produzido efeito mesmo com falha HTTP ou de rede;
    /// preservar sua causa sem autorizar uma segunda tentativa.
    NaoConfirmadoNoEnvio(#[source] ErroSubmissaoFormulario),
    /// O GET de reconciliação falhou após o POST; preservar a causa
    /// observada, sem inferir que o documento deixou de ser criado.
    NaoConfirmadoNaReleitura(#[source] ErroProcessoObter),
}

pub(super) fn postar_e_verificar<T, R, P, L>(
    transporte: &mut T,
    submissao: SubmissaoDerivada,
    cookie: &CookieDaSessao,
    antes: &FotoDaCriacao,
    tipo: &str,
    enviar: P,
    reler: L,
) -> Result<String, ErroConfirmacaoDaCriacao>
where
    R: Read,
    P: FnOnce(&mut T, SubmissaoDerivada, &str) -> Result<RespostaDoPost<R>, FalhaTransporte>,
    L: FnOnce(&mut T) -> Result<FotoDaCriacao, ErroProcessoObter>,
{
    // Nunca emitir uma escrita quando a própria foto inicial já impede
    // demonstrar seu efeito. Validação anterior à primeira requisição.
    if antes.pastas_nao_carregadas {
        return Err(ErroConfirmacaoDaCriacao::ArvoreIncompleta);
    }
    let mut ids = std::collections::HashSet::new();
    if antes.id.is_empty()
        || antes
            .documentos
            .iter()
            .any(|documento| documento.id.is_empty() || !ids.insert(documento.id.as_str()))
    {
        return Err(ErroConfirmacaoDaCriacao::FotoInicialInconsistente);
    }
    let resposta = enviar_submissao_via(submissao, cookie, |post, cookie| {
        enviar(transporte, post, cookie)
    });
    // Sempre fazer uma leitura independente mesmo após erro do POST.
    // Não perder a causa de envio quando a árvore muda concorrentemente
    // ou quando a releitura é inconclusiva.
    let depois = reler(transporte);
    let resposta = match resposta {
        Ok(resposta) => resposta,
        Err(causa) => return Err(ErroConfirmacaoDaCriacao::NaoConfirmadoNoEnvio(causa)),
    };
    let depois = depois.map_err(ErroConfirmacaoDaCriacao::NaoConfirmadoNaReleitura)?;
    if depois.pastas_nao_carregadas || antes.id != depois.id {
        return Err(ErroConfirmacaoDaCriacao::NaoConfirmado);
    }
    let numero = conferir_criacao(&antes.documentos, &depois.documentos, tipo)
        .map_err(ErroConfirmacaoDaCriacao::NaoDemonstradoNaArvore)?;
    let id_novo = depois
        .documentos
        .iter()
        .find(|doc| doc.numero_sei.as_deref() == Some(numero.as_str()))
        .map(|doc| doc.id.as_str())
        .ok_or(ErroConfirmacaoDaCriacao::NaoConfirmado)?;
    // HTML 200 genérico não confirma que o servidor aceitou a gravação.
    // Em conta compartilhada, a mudança da árvore pode ser concorrente.
    // Só o redirecionamento observado para o ID do documento novo serve
    // de indício adicional, ainda condicionado à releitura independente.
    let resposta_conferivel = match &resposta {
        RespostaDoEnvio::Html(_) => false,
        RespostaDoEnvio::Redirecionamento(redirect) => {
            redirect.localizacao().is_some_and(|destino| {
                redirecionamento_do_mesmo_controlador(destino, redirect.url_do_envio(), id_novo)
            })
        }
    };
    if !resposta_conferivel {
        return Err(ErroConfirmacaoDaCriacao::NaoConfirmado);
    }
    Ok(numero)
}

#[cfg(test)]
mod testes_causas {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn falhas_pos_escrita_preservam_causa_no_error_source() {
        let envio = ErroConfirmacaoDaCriacao::NaoConfirmadoNoEnvio(
            ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede),
        );
        assert!(matches!(
            envio
                .source()
                .and_then(|fonte| fonte.downcast_ref::<ErroSubmissaoFormulario>()),
            Some(ErroSubmissaoFormulario::Transporte(FalhaTransporte::Rede))
        ));

        let releitura = ErroConfirmacaoDaCriacao::NaoConfirmadoNaReleitura(
            ErroProcessoObter::Transporte(FalhaTransporte::Rede),
        );
        assert!(matches!(
            releitura
                .source()
                .and_then(|fonte| fonte.downcast_ref::<ErroProcessoObter>()),
            Some(ErroProcessoObter::Transporte(FalhaTransporte::Rede))
        ));
    }
}
