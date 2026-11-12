//! Fluxo de ciência no processo: prévia, único GET de escrita e prova
//! mediante nova leitura da árvore e da Lista de Ciências. Sem retry.
use super::{AlvoDaCiencia, atividade_do_aceite, atividade_na_lista};
use crate::arvore_processo::{ErroCienciaNaArvore, url_da_ciencia};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra;
use crate::transporte::{FalhaTransporte, decodificar_resposta};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CienciaDoProcesso {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) confirmado: bool,
    pub(crate) verificado: bool,
    pub(crate) id_atividade: Option<String>,
    pub(crate) usuario: Option<String>,
    pub(crate) requisicoes: u8,
}

/// Causa original do único GET que pode registrar a ciência do processo.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaDoAceiteProcesso {
    Transporte(#[from] FalhaTransporte),
    /// A mensagem da validação da Infra, numa resposta 200.
    ValidacaoDaInfra(String),
    Aceite(#[from] super::ErroCiencia),
}

/// Por que a releitura não comprovou a ciência do processo depois do GET
/// mutador. Cada etapa guarda o erro original e sobe com `?`.
#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum VerificacaoDaCiencia {
    Abertura(#[from] ErroProcessoObter),
    IdentidadeDoProcessoAlteradaNaReleitura,
    AceiteSemIdDeAtividade,
    Barra(#[from] ErroCienciaNaArvore),
    ListaDeCienciasNaoPublicadaNaArvoreRelida,
    Transporte(#[from] FalhaTransporte),
    Lista(#[from] super::ErroCiencia),
    AtividadeNaoEncontradaNaListaRelida,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCienciaProcesso {
    Abertura(#[from] ErroProcessoObter),
    Barra(#[from] ErroCienciaNaArvore),
    CienciaNaoOferecida,
    UnidadeNaoPublicada,
    OrigemNaoAutorizada,
    Tela(#[from] FalhaTransporte),
    TelaRedirecionada,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
    /// A escrita pode ter ocorrido. NUNCA repetir; revisar a lista manualmente.
    NaoConfirmado {
        motivo: VerificacaoDaCiencia,
        requisicoes: u8,
        falha_do_aceite: Option<FalhaDoAceiteProcesso>,
    },
}

pub(crate) struct PedidoDaCiencia<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) confirmar: bool,
}

/// Prévia: só abertura. Confirmação: um GET de escrita e uma releitura, sem
/// seguir 302 nem repetir o GET mutador mesmo após falha de transporte.
pub(crate) fn ciencia_no_processo<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaCiencia<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<CienciaDoProcesso, ErroCienciaProcesso> {
    let instalacao =
        Instalacao::interpretar(sessao.origem).ok_or(ErroCienciaProcesso::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )?;
    let tela = acoes
        .ciencia?
        .ok_or(ErroCienciaProcesso::CienciaNaoOferecida)?;
    let unidade = aberto
        .unidade
        .ok_or(ErroCienciaProcesso::UnidadeNaoPublicada)?;
    let mut resultado = CienciaDoProcesso {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id,
        unidade,
        confirmado: pedido.confirmar,
        verificado: false,
        id_atividade: None,
        usuario: None,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(resultado);
    }
    let cookie = acoes.cookie.clone();
    let (html, _) = obter_tela(&mut rede, &cookie, tela.as_str()).map_err(|falha| match falha {
        FalhaDaTela::Transporte(e) => ErroCienciaProcesso::Tela(e),
        FalhaDaTela::Redirecionada => ErroCienciaProcesso::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
            ErroCienciaProcesso::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
        }
    })?;
    let url = url_da_ciencia(&instalacao, &html, &resultado.id_procedimento)?;

    // A partir daqui um único GET da própria ciência: qualquer falha exige
    // verificação por leitura, jamais nova tentativa de escrita.
    let aceite = ler_aceite(&mut rede, &cookie, url.as_str(), &resultado.id_procedimento);
    let atividade = aceite.as_ref().ok().map(String::as_str);

    let prova = verificar(
        sessao,
        pedido,
        indice,
        &cookie,
        &mut rede,
        &resultado.id_procedimento,
        atividade,
    );
    resultado.requisicoes = rede.feitas();
    let (atividade, falha_do_aceite) = match aceite {
        Ok(id) => (Some(id), None),
        Err(falha) => (None, Some(falha)),
    };
    match prova {
        Ok((true, usuario)) => {
            resultado.verificado = true;
            resultado.id_atividade = atividade;
            resultado.usuario = usuario;
            Ok(resultado)
        }
        Ok((false, _)) => Err(ErroCienciaProcesso::NaoConfirmado {
            motivo: VerificacaoDaCiencia::AtividadeNaoEncontradaNaListaRelida,
            requisicoes: rede.feitas(),
            falha_do_aceite,
        }),
        Err(motivo) => Err(ErroCienciaProcesso::NaoConfirmado {
            motivo,
            requisicoes: rede.feitas(),
            falha_do_aceite,
        }),
    }
}

/// Nunca repete a escrita: mantém a recusa, o HTTP ou o transporte
/// originais, mesmo quando a releitura posterior não consegue comprovar.
fn ler_aceite<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
    id_procedimento: &str,
) -> Result<String, FalhaDoAceiteProcesso> {
    let resposta = rede.get_sem_seguir(url, cookie)?;
    let html = decodificar_resposta(resposta.resposta)?;
    if let Some(mensagem) = mensagem_de_validacao_da_infra(&html) {
        return Err(FalhaDoAceiteProcesso::ValidacaoDaInfra(mensagem));
    }
    atividade_do_aceite(&html, AlvoDaCiencia::Processo { id_procedimento })
        .map_err(FalhaDoAceiteProcesso::Aceite)
}

/// Mesmo se o GET mutador falhou, reabre UMA vez o processo. Só consulta a
/// lista se o aceite publicou atividade única e o novo link foi observado.
fn verificar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaCiencia<'_>,
    indice: &IndiceLocal,
    cookie: &str,
    rede: &mut Contador<'_, V>,
    id_original: &str,
    atividade: Option<&str>,
) -> Result<(bool, Option<String>), VerificacaoDaCiencia> {
    let (novo, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, false),
        indice,
        rede,
    )?;
    if novo.arvore.processo.id != id_original {
        return Err(VerificacaoDaCiencia::IdentidadeDoProcessoAlteradaNaReleitura);
    }
    let Some(atividade) = atividade else {
        return Err(VerificacaoDaCiencia::AceiteSemIdDeAtividade);
    };
    let lista = acoes
        .lista_ciencias?
        .ok_or(VerificacaoDaCiencia::ListaDeCienciasNaoPublicadaNaArvoreRelida)?;
    let resposta = rede.get_sem_seguir(lista.as_str(), cookie)?;
    let html = decodificar_resposta(resposta.resposta)?;
    let presente = atividade_na_lista(&html, atividade)?;
    let usuario = if presente {
        super::lista_autoria::usuario_da_atividade_na_lista(&html, atividade)?
    } else {
        None
    };
    Ok((presente, usuario))
}
