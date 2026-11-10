//! Andamentos depois da árvore (contracts/cli/processo-obter-andamentos.md).
//! No caminho 1 o link guardado do histórico é tentado primeiro; login,
//! redirect ou página de outro NUP o apagam e o link da árvore recém-lida
//! segue. Status de falha não é palpite: erro de transporte, sem retry.
//! `--andamentos-completos` acrescenta o POST `verHistorico('P')` só se o
//! GET serviu a visão `R`. A visão não é restaurada.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite};
use super::login::{DestinoDoRedirecionamento, destino};
use super::{Andamentos, EnvioDoProcesso, ErroProcessoObter as E, PedidoDeAndamentos};
use crate::formulario_web::{
    ErroSubmissaoFormulario, RespostaDoEnvio, TelaServida, derivar_submissao, enviar_submissao_via,
};
use crate::historico_processo::{
    ACAO_DO_HISTORICO, AndamentosLidos, CAMPO_TIPO, ErroAndamentos, ErroLinkDoHistorico,
    TipoDoHistorico, VER_HISTORICO_COMPLETO, conferir_regra_de_ver_historico, ler_andamentos,
};
use crate::pagina_do_controle::codificacao_da_tela;
use crate::transporte::StatusHttpRecusado;

/// O formulário `verHistorico('P')` não foi derivado nem enviado como
/// observado. Tipo próprio da etapa do histórico completo, distinto da
/// pesquisa rápida; o nome público é o do formulário (`transparente`).
#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum HistoricoNaoPedido {
    #[causa(transparente)]
    Formulario(ErroSubmissaoFormulario),
}

impl HistoricoNaoPedido {
    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Formulario(erro) => erro.status_http(),
        }
    }
}

/// O pedido e o que o fluxo já sabe ao chegar aqui.
pub(super) struct Leitura<'a> {
    pub(super) pedido: PedidoDeAndamentos,
    /// Caminho 1: a árvore veio do índice; o histórico guardado vem antes.
    pub(super) guardado_primeiro: bool,
    pub(super) unidade: Option<&'a str>,
}

/// Página do histórico lida: URL de onde veio, HTML, MIME e a leitura.
struct PaginaLida {
    url: String,
    html: String,
    mime: String,
    lidos: AndamentosLidos,
}

pub(super) fn ler<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    aberta: &ArvoreAberta,
    leitura: &Leitura<'_>,
) -> Result<Andamentos, E> {
    if leitura.pedido == PedidoDeAndamentos::Omitidos {
        return Ok(Andamentos::Omitidos);
    }
    let id = aberta.arvore.processo.id.as_str();
    let guardada = if leitura.guardado_primeiro {
        pelo_guardado(contexto, rede, id)?
    } else {
        None
    };
    let pagina = match guardada {
        Some(pagina) => pagina,
        None => match &aberta.historico {
            Ok(endereco) => pelo_link_da_arvore(contexto, rede, endereco.as_str())?,
            Err(ErroLinkDoHistorico::LinkDoHistoricoAusenteNaArvore) => {
                return Ok(Andamentos::LinkNaoPublicadoNaArvore);
            }
            Err(erro) => return Err(E::LinkDoHistorico(*erro)),
        },
    };
    contexto.aprender_link(ACAO_DO_HISTORICO, id, &pagina.url);
    if leitura.pedido == PedidoDeAndamentos::Completos
        && pagina.lidos.tipo == TipoDoHistorico::Resumido
    {
        return ver_completo(contexto, rede, &pagina, leitura.unidade).map(Andamentos::Lidos);
    }
    Ok(Andamentos::Lidos(pagina.lidos))
}

/// Link guardado nesta geração e unidade. `Ok(None)`: não havia, ou o
/// palpite errou e foi esquecido.
fn pelo_guardado<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    id: &str,
) -> Result<Option<PaginaLida>, E> {
    let Some(link) = contexto
        .chave(ACAO_DO_HISTORICO, id)
        .and_then(|chave| contexto.indice.link(&contexto.instalacao, chave))
    else {
        return Ok(None);
    };
    let url = link.url().to_owned();
    match rede.get(&url)? {
        Lida::Html { html, mime } => match ler_andamentos(&html, contexto.nup) {
            Ok(lidos) => Ok(Some(PaginaLida {
                url,
                html,
                mime,
                lidos,
            })),
            Err(erro) if erro.diverge_do_pedido() => {
                contexto.esquecer_link(ACAO_DO_HISTORICO, id);
                Ok(None)
            }
            Err(erro) => Err(E::Andamentos(erro)),
        },
        Lida::Login | Lida::Redirecionamento(_) => {
            contexto.esquecer_link(ACAO_DO_HISTORICO, id);
            Ok(None)
        }
        Lida::Falha(falha) => Err(E::Transporte(falha)),
    }
}

fn pelo_link_da_arvore<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    url: &str,
) -> Result<PaginaLida, E> {
    match rede.get(url)? {
        Lida::Html { html, mime } => {
            let lidos = ler_andamentos(&html, contexto.nup)?;
            Ok(PaginaLida {
                url: url.to_owned(),
                html,
                mime,
                lidos,
            })
        }
        Lida::Redirecionamento(local) => Err(fora_de_palpite(local.as_deref())),
        Lida::Login => Err(E::SessaoEncerrada),
        Lida::Falha(falha) => Err(E::Transporte(falha)),
    }
}

/// POST `verHistorico('P')` do formulário servido; publica só a visão `P`.
fn ver_completo<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    pagina: &PaginaLida,
    unidade: Option<&str>,
) -> Result<AndamentosLidos, E> {
    conferir_regra_de_ver_historico(&pagina.html)?;
    let codificacao = codificacao_da_tela(&pagina.mime).ok_or(E::Andamentos(
        ErroAndamentos::CharsetDoHistoricoNaoSuportado,
    ))?;
    let tela = TelaServida {
        url: &pagina.url,
        html: &pagina.html,
        codificacao,
    };
    let unidade = contexto.unidade_pedida.or(unidade).unwrap_or_default();
    let submissao = derivar_submissao(
        &VER_HISTORICO_COMPLETO,
        &contexto.instalacao,
        &tela,
        unidade,
        &[(CAMPO_TIPO, "P")],
    )
    .map_err(HistoricoNaoPedido::Formulario)?;
    rede.requisicoes.registrar();
    let envio = &mut *rede.envio;
    let html = match enviar_submissao_via(submissao, contexto.cookie, |s, c| envio.post(s, c)) {
        Ok(RespostaDoEnvio::Html(html)) => html,
        Ok(RespostaDoEnvio::Redirecionamento(r)) => {
            return Err(match destino(r.localizacao()) {
                DestinoDoRedirecionamento::Outro => {
                    E::RedirecionamentoNaoReconhecido(StatusHttpRecusado::sem_mensagem(302))
                }
                _ => E::SessaoEncerrada,
            });
        }
        Err(ErroSubmissaoFormulario::TelaDeLoginRecebidaAposEnvio) => {
            return Err(E::SessaoEncerrada);
        }
        Err(ErroSubmissaoFormulario::Transporte(falha)) => return Err(E::Transporte(falha)),
        Err(outro) => return Err(HistoricoNaoPedido::Formulario(outro).into()),
    };
    let lidos = ler_andamentos(&html, contexto.nup)?;
    if lidos.tipo != TipoDoHistorico::Completo {
        return Err(E::Andamentos(
            ErroAndamentos::VisaoDoHistoricoDiferenteDaPedida,
        ));
    }
    Ok(lidos)
}
