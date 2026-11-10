//! Fluxo de ciência no processo: prévia, único GET de escrita e prova
//! mediante nova leitura da árvore e da Lista de Ciências. Sem retry.
use super::{AlvoDaCiencia, atividade_do_aceite, atividade_na_lista};
use crate::arvore_processo::{ErroCienciaNaArvore, url_da_ciencia};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, decodificar_resposta};

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
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum FalhaDoAceiteProcesso {
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(http, nome = "validacao_da_infra_na_ciencia_do_processo")]
    ValidacaoDaInfra(StatusHttpRecusado),
    #[causa(nome = "atividade_do_aceite_do_processo_nao_reconhecida")]
    Aceite(super::ErroCiencia),
}

impl FalhaDoAceiteProcesso {
    fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(erro) => erro.status_http(),
            Self::ValidacaoDaInfra(status) => Some(status),
            Self::Aceite(_) => None,
        }
    }

    pub(crate) fn detalhe(&self) -> Option<&'static str> {
        match self {
            Self::Aceite(erro) => Some(crate::erro_publico::folha(erro)),
            _ => None,
        }
    }
}

/// Por que a releitura não comprovou a ciência do processo depois do GET
/// mutador. A causa original fica como `fonte`; o nome público é o mesmo.
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum VerificacaoDaCiencia {
    #[causa(fonte, nome = "arvore_nao_relida_depois_da_ciencia")]
    ArvoreNaoRelida(ErroProcessoObter),
    #[causa(nome = "identidade_do_processo_alterada_na_releitura")]
    IdentidadeAlterada,
    #[causa(nome = "aceite_sem_id_de_atividade_depois_da_escrita")]
    AceiteSemAtividade,
    #[causa(fonte, nome = "lista_de_ciencias_fora_da_forma")]
    ListaForaDaForma(ErroCienciaNaArvore),
    #[causa(nome = "lista_de_ciencias_nao_publicada_na_arvore_relida")]
    ListaNaoPublicada,
    #[causa(fonte, nome = "releitura_da_lista_de_ciencias_falhou")]
    ReleituraFalhou(FalhaTransporte),
    #[causa(fonte, nome = "resposta_da_lista_de_ciencias_nao_lida")]
    RespostaNaoLida(FalhaTransporte),
    #[causa(interna)]
    Lista(super::ErroCiencia),
    #[causa(nome = "atividade_da_ciencia_nao_encontrada_na_lista_relida")]
    AtividadeNaoEncontrada,
}

#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum ErroCienciaProcesso {
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    #[causa(interna)]
    Barra(ErroCienciaNaArvore),
    #[causa(nome = "ciencia_nao_oferecida_na_barra")]
    CienciaNaoOferecida,
    #[causa(nome = "unidade_nao_publicada_antes_da_ciencia")]
    UnidadeNaoPublicada,
    OrigemNaoAutorizada,
    #[causa(interna)]
    Cookie(FalhaTransporte),
    #[causa(interna)]
    Tela(FalhaTransporte),
    #[causa(nome = "tela_de_ciencia_redirecionada")]
    TelaRedirecionada,
    #[causa(nome = "sessao_encerrada_rode_ies_login")]
    SessaoEncerrada,
    /// A escrita pode ter ocorrido. NUNCA repetir; revisar a lista manualmente.
    #[causa(nome = "efeito_incerto_na_ciencia")]
    EfeitoIncerto {
        motivo: VerificacaoDaCiencia,
        requisicoes: u8,
        falha_do_aceite: Option<FalhaDoAceiteProcesso>,
    },
}

impl ErroCienciaProcesso {
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(e) => e.mensagem(),
            Self::Barra(_) => "a ação de ciência da barra está fora da forma observada",
            Self::CienciaNaoOferecida => "o processo não oferece ciência na barra",
            Self::UnidadeNaoPublicada => "a abertura não publicou a unidade de trabalho",
            Self::OrigemNaoAutorizada => "a origem da instalação não é autorizada",
            Self::Cookie(_) => "o arquivo privado da sessão não pôde ser lido",
            Self::Tela(_) | Self::TelaRedirecionada => "a tela de ciência não foi servida",
            Self::SessaoEncerrada => "a sessão encerrou; rode ies login",
            Self::EfeitoIncerto { .. } => {
                "a ciência pode ter sido registrada; conferir a Lista de Ciências no SEI antes de nova confirmação"
            }
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http(),
            Self::Cookie(erro) | Self::Tela(erro) => erro.status_http(),
            Self::EfeitoIncerto {
                falha_do_aceite, ..
            } => falha_do_aceite
                .as_ref()
                .and_then(FalhaDoAceiteProcesso::status_http),
            _ => None,
        }
    }

    pub(crate) const fn saida(&self) -> u8 {
        if matches!(self, Self::EfeitoIncerto { .. }) {
            3
        } else {
            1
        }
    }

    pub(crate) const fn requisicoes(&self) -> Option<u8> {
        match self {
            Self::EfeitoIncerto { requisicoes, .. } => Some(*requisicoes),
            _ => None,
        }
    }
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
    use ErroCienciaProcesso as E;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) = abrir_para_escrever(
        sessao,
        &leitura(pedido.nup, pedido.unidade, true),
        indice,
        &mut rede,
    )
    .map_err(E::Abertura)?;
    let tela = acoes
        .ciencia
        .map_err(E::Barra)?
        .ok_or(E::CienciaNaoOferecida)?;
    let unidade = aberto.unidade.ok_or(E::UnidadeNaoPublicada)?;
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
    let cookie = ler_cookie(sessao).map_err(E::Cookie)?;
    let (html, _) = obter_tela(&mut rede, &cookie, tela.as_str()).map_err(|falha| match falha {
        FalhaDaTela::Transporte(e) => E::Tela(e),
        FalhaDaTela::Redirecionada => E::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => E::SessaoEncerrada,
    })?;
    let url = url_da_ciencia(&instalacao, &html, &resultado.id_procedimento).map_err(E::Barra)?;

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
        Ok((false, _)) => Err(E::EfeitoIncerto {
            motivo: VerificacaoDaCiencia::AtividadeNaoEncontrada,
            requisicoes: rede.feitas(),
            falha_do_aceite,
        }),
        Err(motivo) => Err(E::EfeitoIncerto {
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
    let resposta = rede
        .get_sem_seguir(url, cookie)
        .map_err(FalhaDoAceiteProcesso::Transporte)?;
    let html =
        decodificar_resposta(resposta.resposta).map_err(FalhaDoAceiteProcesso::Transporte)?;
    if let Some(mensagem) = mensagem_de_validacao_da_infra(&html) {
        return Err(FalhaDoAceiteProcesso::ValidacaoDaInfra(
            StatusHttpRecusado {
                status: 200,
                mensagem_do_servidor: Some(mensagem),
            },
        ));
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
    )
    .map_err(VerificacaoDaCiencia::ArvoreNaoRelida)?;
    if novo.arvore.processo.id != id_original {
        return Err(VerificacaoDaCiencia::IdentidadeAlterada);
    }
    let Some(atividade) = atividade else {
        return Err(VerificacaoDaCiencia::AceiteSemAtividade);
    };
    let lista = acoes
        .lista_ciencias
        .map_err(VerificacaoDaCiencia::ListaForaDaForma)?
        .ok_or(VerificacaoDaCiencia::ListaNaoPublicada)?;
    let resposta = rede
        .get_sem_seguir(lista.as_str(), cookie)
        .map_err(VerificacaoDaCiencia::ReleituraFalhou)?;
    let html =
        decodificar_resposta(resposta.resposta).map_err(VerificacaoDaCiencia::RespostaNaoLida)?;
    let presente = atividade_na_lista(&html, atividade)?;
    let usuario = if presente {
        super::lista_autoria::usuario_da_atividade_na_lista(&html, atividade)?
    } else {
        None
    };
    Ok((presente, usuario))
}
