//! Ciência do documento, oráculo Todos `tests/test_documento_ciente_010.py`
//! @323eb01: prévia sem mutação, um GET mutador, releitura independente.
//! A lista CD<id> do documento é a única prova, nunca a C do processo.
use super::{AlvoDaCiencia, atividade_do_aceite, atividade_na_lista};
use crate::arvore_processo::{
    ErroBarraDoDocumento, ciencia_do_documento_na_barra, lista_do_documento, tela_do_documento,
    url_da_ciencia_do_documento,
};
use crate::documento::NumeroDocumento;
use crate::erro_html::ErroHtml;
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroDocumentoObter, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    abrir_documento_na_arvore,
};
use crate::transporte::validacao_da_infra::mensagem_de_validacao_da_infra;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado, decodificar_resposta};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CienciaDoDocumento {
    pub(crate) numero: String,
    pub(crate) id_documento: String,
    pub(crate) rotulo: String,
    pub(crate) processo_nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: Option<String>,
    pub(crate) confirmado: bool,
    pub(crate) verificado: bool,
    pub(crate) id_atividade: Option<String>,
    pub(crate) usuario: Option<String>,
    pub(crate) requisicoes: u8,
}

/// Causa tipada da resposta do GET que pode registrar ciência. A decisão de
/// conferir o efeito não elimina o fato observado durante o único envio.
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum FalhaDoAceite {
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(http)]
    ValidacaoDaInfra(StatusHttpRecusado),
    #[causa(interna)]
    Aceite(super::ErroCiencia),
}

impl FalhaDoAceite {
    fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte(e) => e.status_http(),
            Self::ValidacaoDaInfra(status) => Some(status),
            Self::Aceite(_) => None,
        }
    }
}

/// Por que a releitura não comprovou a ciência do documento depois do GET
/// mutador. Cada etapa guarda o erro original e sobe com `?`.
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum VerificacaoDaCienciaDoDocumento {
    #[causa(interna)]
    Abertura(ErroDocumentoObter),
    IdentidadeDoDocumentoAlteradaNaReleitura,
    AceiteSemAtividadeDoDocumento,
    #[causa(transparente)]
    Html(ErroHtml),
    ListaDeCienciasDoDocumentoAusente,
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(interna)]
    Lista(super::ErroCiencia),
    AtividadeNaoEncontradaNaListaDoDocumento,
}

impl VerificacaoDaCienciaDoDocumento {
    fn descricao(&self) -> String {
        match self {
            Self::Abertura(e) => e.mensagem().to_owned(),
            Self::IdentidadeDoDocumentoAlteradaNaReleitura => {
                "IdentidadeDoDocumentoAlteradaNaReleitura".to_owned()
            }
            Self::AceiteSemAtividadeDoDocumento => "AceiteSemAtividadeDoDocumento".to_owned(),
            Self::Html(e) => format!("{e:?}"),
            Self::ListaDeCienciasDoDocumentoAusente => {
                "ListaDeCienciasDoDocumentoAusente".to_owned()
            }
            Self::Transporte(e) => e.mensagem().to_owned(),
            Self::Lista(e) => format!("{e:?}"),
            Self::AtividadeNaoEncontradaNaListaDoDocumento => {
                "AtividadeNaoEncontradaNaListaDoDocumento".to_owned()
            }
        }
    }
}

#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum ErroCienciaDocumento {
    #[causa(interna)]
    Abertura(ErroDocumentoObter),
    #[causa(interna)]
    Barra(ErroBarraDoDocumento),
    #[causa(transparente)]
    Html(ErroHtml),
    CienciaNaoOferecida,
    TelaNaoPublicada,
    OrigemNaoAutorizada,
    #[causa(interna)]
    Cookie(CookieIlegivel),
    #[causa(interna)]
    Tela(FalhaTransporte),
    TelaRedirecionada,
    #[causa(sessao_encerrada)]
    SessaoEncerrada,
    NaoConfirmado {
        motivo: VerificacaoDaCienciaDoDocumento,
        requisicoes: u8,
        falha_do_aceite: Option<FalhaDoAceite>,
    },
}

impl ErroCienciaDocumento {
    pub(crate) fn mensagem(&self) -> String {
        let texto = match self {
            Self::Abertura(e) => e.mensagem(),
            Self::Barra(_) | Self::Html(_) => {
                "a ação de ciência do documento está fora da forma observada"
            }
            Self::CienciaNaoOferecida => "a barra do documento não oferece ciência",
            Self::TelaNaoPublicada => "a árvore não publicou a tela assinada do documento",
            Self::OrigemNaoAutorizada => "a origem da instalação não é autorizada",
            Self::Cookie(_) => "a sessão privada não pôde ser lida",
            Self::Tela(_) | Self::TelaRedirecionada => "a tela do documento não foi servida",
            Self::SessaoEncerrada => "a sessão encerrou; rode ies login",
            Self::NaoConfirmado { .. } => {
                "a ciência pode ter sido registrada; confira a Lista de Ciências do documento no SEI antes de nova confirmação"
            }
        };
        match self {
            Self::NaoConfirmado { motivo, .. } => format!("{texto} ({})", motivo.descricao()),
            _ => texto.to_owned(),
        }
    }
    pub(crate) fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        match self {
            Self::Abertura(e) => e.status_http(),
            Self::Cookie(e) => e.status_http(),
            Self::Tela(e) => e.status_http(),
            Self::NaoConfirmado {
                falha_do_aceite, ..
            } => falha_do_aceite
                .as_ref()
                .and_then(FalhaDoAceite::status_http),
            _ => None,
        }
    }
}

pub(crate) struct PedidoDaCienciaDocumento<'a> {
    pub(crate) numero: &'a NumeroDocumento,
    pub(crate) processo: Option<&'a NumeroProcesso>,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) confirmar: bool,
}

pub(crate) fn ciencia_no_documento<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaCienciaDocumento<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<CienciaDoDocumento, ErroCienciaDocumento> {
    use ErroCienciaDocumento as E;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::OrigemNaoAutorizada)?;
    let mut rede = Contador::novo(envio);
    let abertura = PedidoDoDocumento {
        numero: pedido.numero,
        processo: pedido.processo,
        unidade: pedido.unidade,
        anexo: PedidoDoAnexo::default(),
    };
    let aberto = abrir_documento_na_arvore(sessao, &abertura, indice, &mut rede)?;
    if !ciencia_do_documento_na_barra(&aberto.html, &aberto.id_documento)? {
        return Err(E::CienciaNaoOferecida);
    }
    let mut feito = CienciaDoDocumento {
        numero: aberto.numero,
        id_documento: aberto.id_documento,
        rotulo: aberto.rotulo,
        processo_nup: aberto.processo.nup,
        id_procedimento: aberto.processo.id,
        unidade: aberto.unidade,
        confirmado: pedido.confirmar,
        verificado: false,
        id_atividade: None,
        usuario: None,
        requisicoes: rede.feitas(),
    };
    if !pedido.confirmar {
        return Ok(feito);
    }
    let tela = tela_do_documento(
        &instalacao,
        &aberto.html,
        &feito.id_documento,
        &feito.id_procedimento,
    )
    .ok_or(E::TelaNaoPublicada)?;
    let cookie = ler_cookie(sessao)?;
    let (html, _) = obter_tela(&mut rede, &cookie, tela.as_str()).map_err(|e| match e {
        FalhaDaTela::Transporte(falha) => E::Tela(falha),
        FalhaDaTela::Redirecionada => E::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => E::SessaoEncerrada,
    })?;
    let url = url_da_ciencia_do_documento(
        &instalacao,
        &html,
        &feito.id_documento,
        &feito.id_procedimento,
    )
    .map_err(E::Html)?;
    // Este GET pode gravar a ciência. Após qualquer resposta ou timeout,
    // apenas releitura, nunca segundo GET mutador.
    let aceite = ler_aceite(
        &mut rede,
        &cookie,
        url.as_str(),
        &feito.id_procedimento,
        &feito.id_documento,
    );
    let atividade = aceite.as_ref().ok().map(String::as_str);
    let prova = verificar(
        sessao,
        pedido,
        indice,
        &instalacao,
        &cookie,
        &mut rede,
        &feito.id_procedimento,
        &feito.id_documento,
        atividade,
    );
    let (atividade, falha_do_aceite) = match aceite {
        Ok(id) => (Some(id), None),
        Err(falha) => (None, Some(falha)),
    };
    feito.requisicoes = rede.feitas();
    match prova {
        Ok((true, usuario)) => {
            feito.verificado = true;
            feito.id_atividade = atividade;
            feito.usuario = usuario;
            Ok(feito)
        }
        Ok((false, _)) => Err(E::NaoConfirmado {
            motivo: VerificacaoDaCienciaDoDocumento::AtividadeNaoEncontradaNaListaDoDocumento,
            requisicoes: rede.feitas(),
            falha_do_aceite,
        }),
        Err(motivo) => Err(E::NaoConfirmado {
            motivo,
            requisicoes: rede.feitas(),
            falha_do_aceite,
        }),
    }
}

/// Lê uma vez o GET potencialmente mutador; a causa é mantida para o
/// diagnóstico, mas somente a releitura independente decide o resultado.
fn ler_aceite<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    url: &str,
    id_procedimento: &str,
    id_documento: &str,
) -> Result<String, FalhaDoAceite> {
    let resposta = rede.get_sem_seguir(url, cookie)?;
    let html = decodificar_resposta(resposta.resposta)?;
    if let Some(mensagem) = mensagem_de_validacao_da_infra(&html) {
        return Err(FalhaDoAceite::ValidacaoDaInfra(StatusHttpRecusado {
            status: 200,
            mensagem_do_servidor: Some(mensagem),
        }));
    }
    atividade_do_aceite(
        &html,
        AlvoDaCiencia::Documento {
            id_procedimento,
            id_documento,
        },
    )
    .map_err(FalhaDoAceite::Aceite)
}

#[allow(clippy::too_many_arguments)]
fn verificar<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaCienciaDocumento<'_>,
    indice: &IndiceLocal,
    instalacao: &Instalacao,
    cookie: &str,
    rede: &mut Contador<'_, V>,
    id_procedimento: &str,
    id_documento: &str,
    atividade: Option<&str>,
) -> Result<(bool, Option<String>), VerificacaoDaCienciaDoDocumento> {
    let abertura = PedidoDoDocumento {
        numero: pedido.numero,
        processo: pedido.processo,
        unidade: pedido.unidade,
        anexo: PedidoDoAnexo::default(),
    };
    let novo = abrir_documento_na_arvore(sessao, &abertura, indice, rede)?;
    if novo.id_documento != id_documento || novo.processo.id != id_procedimento {
        return Err(VerificacaoDaCienciaDoDocumento::IdentidadeDoDocumentoAlteradaNaReleitura);
    }
    let Some(atividade) = atividade else {
        return Err(VerificacaoDaCienciaDoDocumento::AceiteSemAtividadeDoDocumento);
    };
    let lista = lista_do_documento(instalacao, &novo.html, id_procedimento, id_documento)?
        .ok_or(VerificacaoDaCienciaDoDocumento::ListaDeCienciasDoDocumentoAusente)?;
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
