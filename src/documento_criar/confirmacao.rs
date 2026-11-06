//! Fronteira de escrita única e releitura para documento criar (#246).
//! O transporte é injetável; a confirmação nunca repete a submissão.
use crate::arvore_processo::{ArvoreLida, DocumentoNaArvore};
use crate::documento_criar::conferir_criacao;
use crate::documento_criar::fluxo::ErroPreviaCriacao;
use crate::formulario_web::{RespostaDoEnvio, enviar_submissao_via};
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

/// O 302 não é seguido, mas só serve de indício se seu destino
/// continuar no controlador HTTPS da própria instalação. Um Location
/// externo/inválido pode coincidir com criação concorrente na árvore.
fn redirecionamento_do_mesmo_controlador(localizacao: &str, url_do_envio: &str) -> bool {
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
            .or_else(|| {
                localizacao.strip_prefix(&format!("/{}?", origem.caminho))
            })
    };
    consulta.is_some_and(|consulta| {
        consulta
            .split('&')
            .filter_map(|campo| campo.strip_prefix("acao="))
            .filter(|acao| !acao.is_empty())
            .count()
            == 1
    })
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
    // Nunca emitir uma escrita quando a própria foto inicial já impede
    // demonstrar seu efeito. Validação anterior à primeira requisição.
    if antes.pastas_nao_carregadas {
        return Err(ErroPreviaCriacao::ArvoreIncompleta);
    }
    let mut ids = std::collections::HashSet::new();
    if antes.id.is_empty()
        || antes
            .documentos
            .iter()
            .any(|documento| documento.id.is_empty() || !ids.insert(documento.id.as_str()))
    {
        return Err(ErroPreviaCriacao::FotoInicialInconsistente);
    }
    let resposta = enviar_submissao_via(submissao, cookie, |post, cookie| {
        enviar(transporte, post, cookie)
    });
    let depois = reler(transporte).map_err(|()| ErroPreviaCriacao::EfeitoIncerto)?;
    if depois.pastas_nao_carregadas || antes.id != depois.id {
        return Err(ErroPreviaCriacao::EfeitoIncerto);
    }
    let numero = conferir_criacao(&antes.documentos, &depois.documentos, tipo)
        .map_err(|_| ErroPreviaCriacao::EfeitoIncerto)?;
    // 302 sem Location não é uma resposta de navegação válida.
    // Mesmo que a árvore tenha mudado, pode ser escrita concorrente;
    // não atribuir essa alteração a um POST de resultado inconclusivo.
    let resposta_conferivel = match &resposta {
        Ok(RespostaDoEnvio::Html(_)) => true,
        Ok(RespostaDoEnvio::Redirecionamento(redirect)) => redirect
            .localizacao()
            .is_some_and(|destino| {
                redirecionamento_do_mesmo_controlador(destino, redirect.url_do_envio())
            }),
        Err(_) => false,
    };
    if !resposta_conferivel {
        return Err(ErroPreviaCriacao::EfeitoIncerto);
    }
    Ok(numero)
}
