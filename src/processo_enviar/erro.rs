//! Recusas e resultado de `ies processo enviar`
//! (contracts/cli/processo-enviar.md). Uma variante por fato.
use crate::arvore_processo::{ErroEnviarNaBarra, MotivoAbertoEm};
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_concluir::{FalhaDoEnvio, VerificacaoPorLeitura};
use crate::processo_obter::ErroProcessoObter;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

/// Recusas da tela "Enviar Processo", antes de qualquer POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroTelaDeEnvio {
    #[causa(nome = "tela_de_envio_nao_interpretada")]
    HtmlNaoInterpretado,
    FormularioDeEnvioAusente,
    CampoNaoPrevistoNaTelaDeEnvio,
    TelaDeEnvioDeOutroProcesso,
    TelaDeEnvioComOpcoesPreenchidas,
    OrgaoDaTelaDeEnvioNaoSelecionado,
    AutocompletarDeUnidadeNaoDeclarado,
    ConsultaDoAutocompletarForaDaFormaObservada,
}

impl ErroTelaDeEnvio {
    pub(crate) fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
}

/// Por que a consulta do autocomplete não deu a lista de unidades.
#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum FalhaDaConsulta {
    /// Derivação ou cookie: nada saiu.
    #[causa(interna)]
    Derivacao(ErroSubmissaoFormulario),
    /// Rede, tempo, status, tamanho, charset.
    #[causa(interna)]
    Transporte(FalhaTransporte),
    /// 200 sem `application/xml`/`text/xml` (tela de login, outra página).
    RespostaDoAutocompletarNaoXml,
    /// XML sem um `<itens>` único na forma observada.
    #[causa(nome = "resposta_do_autocompletar_fora_da_forma_observada")]
    RespostaDoAutocompletarForaDaForma,
}

impl FalhaDaConsulta {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }

    fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Derivacao(erro) => erro.status_http().cloned(),
            Self::Transporte(falha) => falha.status_http().cloned(),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ErroEnviar {
    /// A abertura do processo (a de `ies processo NUP`) falhou.
    Abertura(ErroProcessoObter),
    UnidadeDaSessaoNaoPublicada,
    SituacaoDoProcessoNaoLidaAntesDeEnviar {
        motivo: MotivoAbertoEm,
    },
    ProcessoNaoAbertoNaUnidadeDaSessao {
        aberto_em: Vec<String>,
    },
    /// A barra oferece enviar, mas `aberto_em` não lista a unidade.
    UnidadeDaSessaoAusenteDaSituacao {
        aberto_em: Vec<String>,
    },
    EnvioNaoPublicadoNaBarra {
        aberto_em: Vec<String>,
    },
    Barra(ErroEnviarNaBarra),
    TelaDaEscrita(FalhaTransporte),
    TelaDaEscritaRedirecionada,
    SessaoEncerrada,
    CharsetDaTelaNaoSuportado,
    TelaDeEnvio(ErroTelaDeEnvio),
    Consulta(FalhaDaConsulta),
    DestinoNaoOferecidoPeloSei {
        oferecidos: Vec<String>,
    },
    VariosDestinosComASigla {
        candidatos: Vec<String>,
    },
    ProcessoJaAbertoNoDestino {
        aberto_em: Vec<String>,
    },
    /// Derivação do POST ou falha antes de ele sair (cookie).
    Formulario(ErroSubmissaoFormulario),
    /// A leitura depois do envio não lista o destino.
    EnvioNaoAplicado {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// O destino está aberto, mas a unidade da sessão não ficou como pedido.
    SituacaoDaOrigemDiferenteDaPedidaAposEnvio {
        aberto_em: Vec<String>,
        depois: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
    },
    /// Saída 3: o POST saiu e a leitura não decide o efeito.
    EfeitoIncertoNoEnvio {
        aberto_em: Vec<String>,
        falha_do_envio: Option<FalhaDoEnvio>,
        verificacao: Box<VerificacaoPorLeitura>,
    },
}

impl ErroEnviar {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.subcodigo(),
            Self::UnidadeDaSessaoNaoPublicada => "unidade_da_sessao_nao_publicada_na_abertura",
            Self::SituacaoDoProcessoNaoLidaAntesDeEnviar { .. } => {
                "situacao_do_processo_nao_lida_antes_de_enviar"
            }
            Self::ProcessoNaoAbertoNaUnidadeDaSessao { .. } => {
                "processo_nao_aberto_na_unidade_da_sessao"
            }
            Self::UnidadeDaSessaoAusenteDaSituacao { .. } => {
                "unidade_da_sessao_ausente_da_situacao"
            }
            Self::EnvioNaoPublicadoNaBarra { .. } => "envio_nao_publicado_na_barra",
            Self::Barra(erro) => erro.subcodigo(),
            Self::TelaDaEscrita(falha) => falha.subcodigo(),
            Self::TelaDaEscritaRedirecionada => "tela_de_envio_redirecionada",
            Self::SessaoEncerrada => "sessao_encerrada_rode_ies_login",
            Self::CharsetDaTelaNaoSuportado => "charset_da_tela_de_envio_nao_suportado",
            Self::TelaDeEnvio(erro) => erro.subcodigo(),
            Self::Consulta(falha) => falha.subcodigo(),
            Self::DestinoNaoOferecidoPeloSei { .. } => "destino_nao_oferecido_pelo_sei",
            Self::VariosDestinosComASigla { .. } => "varios_destinos_com_a_sigla",
            Self::ProcessoJaAbertoNoDestino { .. } => "processo_ja_aberto_no_destino",
            Self::Formulario(erro) => erro.subcodigo(),
            Self::EnvioNaoAplicado { .. } => "envio_nao_aplicado",
            Self::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { .. } => {
                "situacao_da_origem_diferente_da_pedida_apos_envio"
            }
            Self::EfeitoIncertoNoEnvio { .. } => "efeito_incerto_no_envio",
        }
    }

    /// Categoria pública: o próximo passo de quem chama.
    pub(crate) fn codigo(&self) -> &'static str {
        match self {
            Self::EnvioNaoAplicado { .. } => "escrita_nao_aplicada",
            Self::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { .. } => {
                "escrita_diferente_da_solicitada"
            }
            Self::EfeitoIncertoNoEnvio { .. } => "efeito_incerto",
            _ => "fonte_invalida",
        }
    }

    /// 3 só para efeito incerto; as demais recusas saem com 1.
    pub(crate) fn saida(&self) -> u8 {
        match self {
            Self::EfeitoIncertoNoEnvio { .. } => 3,
            _ => 1,
        }
    }

    pub(crate) fn status_http(&self) -> Option<StatusHttpRecusado> {
        match self {
            Self::Abertura(erro) => erro.status_http().cloned(),
            Self::TelaDaEscrita(falha) => falha.status_http().cloned(),
            Self::TelaDaEscritaRedirecionada => Some(StatusHttpRecusado::sem_mensagem(302)),
            Self::Consulta(falha) => falha.status_http(),
            Self::Formulario(erro) => erro.status_http().cloned(),
            Self::EnvioNaoAplicado { falha_do_envio, .. }
            | Self::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { falha_do_envio, .. }
            | Self::EfeitoIncertoNoEnvio { falha_do_envio, .. } => {
                falha_do_envio.as_ref().and_then(FalhaDoEnvio::status_http)
            }
            _ => None,
        }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.mensagem(),
            Self::UnidadeDaSessaoNaoPublicada => {
                "a abertura não publicou a unidade da sessão; nada foi enviado"
            }
            Self::SituacaoDoProcessoNaoLidaAntesDeEnviar { .. } => {
                "a árvore não publica onde o processo está aberto, e o envio não poderia ser conferido; nada foi enviado"
            }
            Self::ProcessoNaoAbertoNaUnidadeDaSessao { .. } => {
                "o processo não está aberto na unidade da sessão; nada foi enviado"
            }
            Self::UnidadeDaSessaoAusenteDaSituacao { .. } => {
                "a barra oferece enviar, mas a situação não lista a unidade da sessão; nada foi enviado"
            }
            Self::EnvioNaoPublicadoNaBarra { .. } => {
                "o processo está aberto na unidade, mas a barra não oferece enviar; nada foi enviado"
            }
            Self::Barra(_) => {
                "o link de enviar da barra não está na forma observada; nada foi enviado"
            }
            Self::TelaDaEscrita(_) | Self::TelaDaEscritaRedirecionada => {
                "a tela Enviar Processo não foi servida; nada foi enviado"
            }
            Self::SessaoEncerrada => "a sessão encerrou; rode `ies login` (nada foi enviado)",
            Self::CharsetDaTelaNaoSuportado => {
                "a tela de envio declara um charset não suportado; nada foi enviado"
            }
            Self::TelaDeEnvio(_) => {
                "a tela Enviar Processo não está na forma observada; nada foi enviado"
            }
            Self::Consulta(_) => {
                "a consulta de unidades da tela de envio não respondeu na forma observada; nada foi enviado"
            }
            Self::DestinoNaoOferecidoPeloSei { .. } => {
                "o SEI não oferece unidade de destino com essa sigla; nada foi enviado"
            }
            Self::VariosDestinosComASigla { .. } => {
                "o SEI oferece mais de uma unidade com essa sigla; nada foi enviado"
            }
            Self::ProcessoJaAbertoNoDestino { .. } => {
                "o processo já está aberto na unidade de destino; nada foi enviado"
            }
            Self::Formulario(_) => "o envio não pôde ser montado; nada foi enviado",
            Self::EnvioNaoAplicado { .. } => {
                "a árvore lida depois de enviar não lista o destino: o processo não foi enviado"
            }
            Self::SituacaoDaOrigemDiferenteDaPedidaAposEnvio { .. } => {
                "o processo foi enviado, mas a unidade da sessão não ficou como pedido (manter aberto)"
            }
            Self::EfeitoIncertoNoEnvio { .. } => {
                "não se sabe se o processo foi enviado; confira com `ies processo NUP` antes de repetir"
            }
        }
    }
}
