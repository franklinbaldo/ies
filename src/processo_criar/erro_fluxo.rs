//! Erros tipados do fluxo de criação; o envio e a releitura preservam as causas.
use super::reconciliacao;
use crate::erro_html::ErroHtml;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_criar_cadastro::ErroCadastro;
use crate::processo_criar_hipotese::ErroHipotese;
use crate::processo_criar_interessados::ErroInteressado;
use crate::processo_criar_lista::ErroListaDeTipos;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

#[derive(Debug, Clone, Copy, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum CausaRespostaCriadora {
    #[causa(nome = "resposta_inesperada_apos_post_criador")]
    RespostaInesperada,
    #[causa(nome = "localizacao_invalida_apos_post_criador")]
    LocalizacaoInvalida,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticoPosEscrita {
    pub(crate) causa_post: &'static str,
    pub(crate) causa_releitura: Option<&'static str>,
    pub(crate) candidatos_observados: Option<Vec<String>>,
    /// NUPs cuja identidade, tipo e especificação foram todos conferidos.
    pub(crate) candidatos_compativeis: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ErroCriar {
    /// Falha antes da escrita; guarda o subcódigo do transporte e, quando
    /// houve, o status e a mensagem saneada do SEI.
    Transporte {
        subcodigo: &'static str,
        http: Option<StatusHttpRecusado>,
    },
    UnidadeDiferente,
    Menu(ErroHtml),
    Lista(ErroListaDeTipos),
    Cadastro(ErroCadastro),
    Formulario,
    TipoSemIdentificador,
    EscolhaInesperada,
    Interessado(ErroInteressado),
    Hipotese(ErroHipotese),
    Fotografia(reconciliacao::ErroFotografia),
    Conferencia(reconciliacao::desfechos::ErroConferencia),
    Tela(crate::arvore_processo::ErroTelaDoProcesso),
    Arvore(crate::arvore_processo::ErroArvore),
    Propriedades(crate::propriedades_processo::ErroPropriedades),
    PropriedadesNaoOferecidas,
    /// Recusa explícita do SEI: não atribuir processos concorrentes a este POST.
    RecusaDoPost(ErroSubmissaoFormulario),
    /// O POST foi tentado; uma leitura única do Controle apurou apenas
    /// candidatos, nunca causalidade. As duas falhas são preservadas.
    PosEscrita {
        causa_post: ErroSubmissaoFormulario,
        posterior: Result<Vec<reconciliacao::CandidatoGerado>, Box<ErroCriar>>,
        correspondentes: Option<Vec<String>>,
    },
    /// O POST recebeu resposta sem prova de identidade; uma releitura
    /// única do Controle preserva o fato da resposta e a falha posterior.
    PosRespostaInesperada {
        causa: CausaRespostaCriadora,
        posterior: Result<Vec<reconciliacao::CandidatoGerado>, Box<ErroCriar>>,
        correspondentes: Option<Vec<String>>,
    },
    EfeitoIncerto,
}

impl ErroCriar {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Transporte { subcodigo, .. } => subcodigo,
            Self::UnidadeDiferente => "unidade_da_criacao_divergente",
            Self::Menu(erro) => erro.onde,
            Self::Lista(erro) => erro.subcodigo(),
            Self::Cadastro(erro) => erro.subcodigo(),
            Self::Formulario => "formulario_de_criacao_nao_reconhecido",
            Self::TipoSemIdentificador => "tipo_de_processo_sem_identificador",
            Self::EscolhaInesperada => "cadastro_nao_devolvido_apos_escolha",
            Self::Interessado(erro) => erro.subcodigo(),
            Self::Hipotese(erro) => erro.subcodigo(),
            Self::Fotografia(erro) => crate::erro_publico::folha(erro),
            Self::Conferencia(erro) => crate::erro_publico::folha(erro),
            Self::Tela(erro) => crate::erro_publico::folha(erro),
            Self::Arvore(erro) => crate::erro_publico::folha(erro),
            Self::Propriedades(erro) => crate::erro_publico::folha(erro),
            Self::PropriedadesNaoOferecidas => "propriedades_nao_oferecidas_ao_candidato",
            Self::RecusaDoPost(erro) => erro.subcodigo(),
            Self::PosEscrita {
                correspondentes: Some(_),
                ..
            }
            | Self::PosRespostaInesperada {
                correspondentes: Some(_),
                ..
            } => "candidatos_ambiguos_na_criacao",
            Self::PosEscrita { .. } | Self::PosRespostaInesperada { .. } | Self::EfeitoIncerto => {
                "efeito_incerto_na_criacao"
            }
        }
    }

    pub(crate) fn opcoes_para_escolha(&self) -> Option<&[(String, String)]> {
        match self {
            Self::Hipotese(erro) => erro.opcoes_para_escolha(),
            _ => None,
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Transporte { http, .. } => http.as_ref(),
            Self::Hipotese(erro) => erro.status_http(),
            Self::PosEscrita { causa_post, .. } => causa_post.status_http(),
            Self::RecusaDoPost(erro) => erro.status_http(),
            _ => None,
        }
    }

    pub(super) fn de_transporte(falha: &FalhaTransporte) -> Self {
        Self::Transporte {
            subcodigo: falha.subcodigo(),
            http: falha.status_http().cloned(),
        }
    }

    pub(super) fn local() -> Self {
        Self::Transporte {
            subcodigo: "falha_de_transporte_na_criacao",
            http: None,
        }
    }

    /// Só a validação explícita da Infra prova que o formulário foi
    /// recusado pelo próprio SEI. HTTP 4xx pode vir de intermediário e não
    /// atesta a ausência do efeito; Gerados vazio tampouco é prova.
    pub(crate) fn nao_aplicada_comprovada(&self) -> bool {
        matches!(
            self,
            Self::RecusaDoPost(ErroSubmissaoFormulario::ValidacaoRecusadaPeloSei(_))
        )
    }

    pub(crate) fn saida(&self) -> u8 {
        if matches!(
            self,
            Self::EfeitoIncerto | Self::PosEscrita { .. } | Self::PosRespostaInesperada { .. }
        ) {
            3
        } else {
            1
        }
    }

    /// O retorno não infere ausência quando não houve candidato novo.
    /// Só expõe subcódigos tipados e NUPs opacos, nunca URLs ou cookies.
    pub(crate) fn diagnostico_apos_post(&self) -> Option<DiagnosticoPosEscrita> {
        let (causa_post, posterior, correspondentes) = match self {
            Self::PosEscrita {
                causa_post,
                posterior,
                correspondentes,
            } => (causa_post.subcodigo(), posterior, correspondentes),
            Self::PosRespostaInesperada {
                causa,
                posterior,
                correspondentes,
            } => (
                crate::erro_publico::folha(causa),
                posterior,
                correspondentes,
            ),
            _ => return None,
        };
        Some(DiagnosticoPosEscrita {
            causa_post,
            causa_releitura: posterior.as_ref().err().map(|erro| erro.subcodigo()),
            candidatos_observados: posterior
                .as_ref()
                .ok()
                .map(|candidatos| candidatos.iter().map(|c| c.nup.clone()).collect()),
            candidatos_compativeis: correspondentes.clone(),
        })
    }
}
