//! Erros tipados do fluxo de criação; o envio e a releitura preservam as causas.
use super::reconciliacao;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_criar_cadastro::ErroCadastro;
use crate::processo_criar_hipotese::ErroHipotese;
use crate::processo_criar_interessados::ErroInteressado;
use crate::processo_criar_lista::ErroListaDeTipos;
use crate::transporte::FalhaTransporte;
#[cfg(test)]
use crate::transporte::StatusHttpRecusado;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum CausaRespostaCriadora {
    RespostaInesperada,
    LocalizacaoInvalida,
}

/// Fato observado na verificação independente após Location legítimo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ProvaDaCriacao {
    PropriedadesDivergentes,
    IdentidadePreexistente,
}

/// Classificacao por fatos tipados, sem interpretar textos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EvidenciaDeCoincidencia {
    UnicaSemProvaCausal,
    Multiplas,
    Parcial,
}

/// Projeção do erro pós-escrita para os testes de reconciliação; a CLI
/// publica o próprio erro tipado.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DiagnosticoPosEscrita {
    pub(crate) evidencia: Option<EvidenciaDeCoincidencia>,
    pub(crate) causa_post: String,
    pub(crate) causa_releitura: Option<String>,
    /// Falha HTTP da leitura de reconciliação, distinta da resposta do POST.
    pub(crate) http_releitura: Option<StatusHttpRecusado>,
    pub(crate) candidatos_observados: Option<Vec<String>>,
    /// NUPs com identidade, tipo e especificação lidos e coincidentes.
    /// Quando a releitura falhou, são provas parciais, não resultado final.
    pub(crate) candidatos_compativeis: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCriar {
    /// O tokenizador de atributos recusou a tag.
    Atributo(#[source] crate::html_atributos::ErroAtributoHtml),
    /// Falha de transporte antes da escrita, com a causa original.
    Transporte(FalhaTransporte),
    /// O envio do formulário falhou antes de qualquer POST tentado.
    Submissao(ErroSubmissaoFormulario),
    /// Origem ou cookie da sessão local ilegíveis, antes da rede.
    SessaoLocalIlegivel,
    UnidadeDiferente,
    Menu(crate::processo_criar_menu::ErroMenu),
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
    ProvaDaCriacao(ProvaDaCriacao),
    /// Falha de leitura após Location legítimo do POST criador.
    /// A causa é da verificação; a criação já pode ter sido aplicada.
    VerificacaoAposPost(Box<ErroCriar>),
    /// Validação expressa da Infra: o próprio SEI publicou não aplicação.
    RecusaDoPost(ErroSubmissaoFormulario),
    /// HTTP 4xx após o POST não prova se a escrita teve efeito: falhar
    /// fechado, sem reconciliar com criação possivelmente concorrente.
    RespostaHttpSemProva(ErroSubmissaoFormulario),
    /// O POST foi tentado; uma leitura única do Controle apurou apenas
    /// candidatos, nunca causalidade. As duas falhas são preservadas.
    PosEscrita {
        causa_post: ErroSubmissaoFormulario,
        posterior: Result<Vec<reconciliacao::CandidatoGerado>, Box<ErroCriar>>,
        correspondentes: Option<Vec<String>>,
        /// Fotografia posterior já lida quando uma prova individual falha.
        observados_antes_da_falha: Option<Box<[reconciliacao::CandidatoGerado]>>,
    },
    /// O POST recebeu resposta sem prova de identidade; uma releitura
    /// única do Controle preserva o fato da resposta e a falha posterior.
    PosRespostaInesperada {
        causa: CausaRespostaCriadora,
        posterior: Result<Vec<reconciliacao::CandidatoGerado>, Box<ErroCriar>>,
        correspondentes: Option<Vec<String>>,
        observados_antes_da_falha: Option<Box<[reconciliacao::CandidatoGerado]>>,
    },
    NaoConfirmado,
}

/// Consultas dos testes sobre o erro tipado; a CLI publica o próprio erro.
#[cfg(test)]
impl ErroCriar {
    /// Somente valores numéricos do atributo maxlength observado na tela;
    /// não publica a descrição nem pressupõe limite global.
    pub(crate) fn limite_da_especificacao(&self) -> Option<(usize, usize)> {
        match self {
            Self::Cadastro(ErroCadastro::EspecificacaoAcimaDoLimite { limite, tamanho }) => {
                Some((*limite, *tamanho))
            }
            _ => None,
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
            Self::Transporte(falha) => falha.status_http(),
            Self::VerificacaoAposPost(erro) => erro.status_http(),
            Self::Hipotese(erro) => erro.status_http(),
            Self::Interessado(erro) => erro.status_http(),
            Self::PosEscrita {
                causa_post,
                posterior,
                ..
            } => causa_post
                .status_http()
                .or_else(|| posterior.as_ref().err().and_then(|erro| erro.status_http())),
            Self::PosRespostaInesperada { posterior, .. } => {
                posterior.as_ref().err().and_then(|erro| erro.status_http())
            }
            Self::Submissao(erro) | Self::RecusaDoPost(erro) | Self::RespostaHttpSemProva(erro) => {
                erro.status_http()
            }
            _ => None,
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

    pub(crate) fn apos_post(&self) -> bool {
        matches!(
            self,
            Self::NaoConfirmado
                | Self::VerificacaoAposPost(_)
                | Self::RespostaHttpSemProva(_)
                | Self::PosEscrita { .. }
                | Self::PosRespostaInesperada { .. }
        )
    }

    /// O retorno não infere ausência quando não houve candidato novo.
    /// Só expõe erros tipados e NUPs opacos, nunca URLs ou cookies.
    pub(crate) fn diagnostico_apos_post(&self) -> Option<DiagnosticoPosEscrita> {
        let (causa_post, posterior, correspondentes, observados) = match self {
            Self::PosEscrita {
                causa_post,
                posterior,
                correspondentes,
                observados_antes_da_falha,
            } => (
                format!("{causa_post:?}"),
                posterior,
                correspondentes,
                observados_antes_da_falha,
            ),
            Self::PosRespostaInesperada {
                causa,
                posterior,
                correspondentes,
                observados_antes_da_falha,
            } => (
                format!("{causa:?}"),
                posterior,
                correspondentes,
                observados_antes_da_falha,
            ),
            _ => return None,
        };
        let evidencia = correspondentes.as_ref().map(|nups| {
            if posterior.is_err() {
                EvidenciaDeCoincidencia::Parcial
            } else if nups.len() == 1 {
                EvidenciaDeCoincidencia::UnicaSemProvaCausal
            } else {
                EvidenciaDeCoincidencia::Multiplas
            }
        });
        Some(DiagnosticoPosEscrita {
            evidencia,
            causa_post,
            causa_releitura: posterior.as_ref().err().map(|erro| format!("{erro:?}")),
            http_releitura: posterior
                .as_ref()
                .err()
                .and_then(|erro| erro.status_http())
                .cloned(),
            candidatos_observados: posterior
                .as_ref()
                .ok()
                .map(Vec::as_slice)
                .or(observados.as_deref())
                .map(|candidatos| candidatos.iter().map(|c| c.nup.clone()).collect()),
            candidatos_compativeis: correspondentes.clone(),
        })
    }
}
