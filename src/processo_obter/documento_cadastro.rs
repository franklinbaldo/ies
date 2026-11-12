//! Cadastro compartilhado: propriedades e anexo externo sem .src (#60).
use super::{ErroDocumentoObter, fora_de_palpite};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::EnderecoDoConteudo;
use crate::instalacao::Instalacao;
use crate::processo_obter::EnvioDoProcesso;
use crate::processo_obter::ErroProcessoObter;
use crate::processo_obter::abertura::{Lida, Rede};
use crate::propriedades_documento::{self, PropriedadesDocumento};

pub(super) struct Cadastro {
    pub(super) propriedades: PropriedadesDocumento,
    pub(super) anexo: Option<EnderecoDoConteudo>,
}

pub(super) fn ler<V: EnvioDoProcesso>(
    instalacao: &Instalacao,
    rede: &mut Rede<'_, V>,
    arvore: &str,
    documento: &str,
    processo: &str,
    pedir_propriedades: bool,
    buscar_anexo: bool,
) -> Result<Cadastro, ErroDocumentoObter> {
    if !pedir_propriedades && !buscar_anexo {
        return Ok(Cadastro {
            propriedades: PropriedadesDocumento::NaoPedidas,
            anexo: None,
        });
    }
    // Sem link de cadastro na barra, ou sem anexo publicado: o pedido do
    // usuário não se cumpre. O anexo é o conteúdo; as propriedades, o cadastro.
    let conteudo_nao_publicado = || ErroDocumentoObter::ConteudoNaoPublicadoNaArvore {
        requisicoes: rede.requisicoes.total_u8(),
    };
    let Some(link) = propriedades_documento::ler_link(instalacao, arvore, documento, processo)
        .map_err(ErroDocumentoObter::Cadastro)?
    else {
        return Err(if buscar_anexo {
            conteudo_nao_publicado()
        } else {
            ErroDocumentoObter::PropriedadesNaoPublicadasNaBarra
        });
    };
    if buscar_anexo
        && !matches!(
            link.fonte,
            AcaoDoSei::DocumentoAlterarRecebido | AcaoDoSei::DocumentoConsultarRecebido
        )
    {
        return Err(conteudo_nao_publicado());
    }
    let html = match rede.get(&link.url).map_err(ErroDocumentoObter::Processo)? {
        Lida::Html { html, .. } => html,
        Lida::Login => {
            return Err(ErroDocumentoObter::Processo(
                ErroProcessoObter::SessaoEncerrada(crate::erro_publico::SessaoEncerrada),
            ));
        }
        Lida::Redirecionamento(local) => {
            return Err(ErroDocumentoObter::Processo(fora_de_palpite(
                local.as_deref(),
            )));
        }
        Lida::Falha(falha) => {
            return Err(ErroDocumentoObter::Processo(ErroProcessoObter::Transporte(
                falha,
            )));
        }
    };
    let mut lidas = propriedades_documento::ler_formulario(&html, documento, processo)
        .map_err(ErroDocumentoObter::Cadastro)?;
    lidas.fonte = link.fonte;
    let anexo = if buscar_anexo {
        propriedades_documento::ler_anexo_publicado(instalacao, &html, documento, processo)
            .map_err(ErroDocumentoObter::Cadastro)?
    } else {
        None
    };
    Ok(Cadastro {
        propriedades: if pedir_propriedades {
            PropriedadesDocumento::Lidas(lidas)
        } else {
            PropriedadesDocumento::NaoPedidas
        },
        anexo,
    })
}
