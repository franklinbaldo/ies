//! Erros tipados do fluxo de criação; o envio e a releitura preservam as causas.
use super::reconciliacao;
use crate::arquivo_privado::ErroArquivoPrivado;
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
#[cfg(test)]
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
    pub(crate) causa_post: String,
    /// Só existe quando a releitura de reconciliação falhou.
    pub(crate) releitura_falhou: Option<FalhaDaReleitura>,
    pub(crate) candidatos_observados: Option<Vec<String>>,
    /// NUPs com identidade, tipo e especificação lidos e coincidentes.
    /// Quando a releitura falhou, são provas parciais, não resultado final.
    pub(crate) candidatos_compativeis: Option<Vec<String>>,
}

/// A releitura que falhou: a causa, e o HTTP dela quando houve status.
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FalhaDaReleitura {
    pub(crate) causa: String,
    /// Falha HTTP da leitura de reconciliação, distinta da resposta do POST.
    pub(crate) http: Option<StatusHttpRecusado>,
}

#[cfg(test)]
impl DiagnosticoPosEscrita {
    /// Os candidatos compatíveis, com a releitura falha como prova parcial.
    pub(crate) fn evidencia(&self) -> Option<EvidenciaDeCoincidencia> {
        self.candidatos_compativeis.as_ref().map(|nups| {
            if self.releitura_falhou.is_some() {
                EvidenciaDeCoincidencia::Parcial
            } else if nups.len() == 1 {
                EvidenciaDeCoincidencia::UnicaSemProvaCausal
            } else {
                EvidenciaDeCoincidencia::Multiplas
            }
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroCriar {
    /// Falha de transporte antes da escrita, com a causa original.
    Transporte(FalhaTransporte),
    /// O envio do formulário falhou antes de qualquer POST tentado.
    Submissao(ErroSubmissaoFormulario),
    /// Origem ou cookie da sessão local ilegíveis, antes da rede.
    SessaoLocalIlegivel,
    /// O arquivo do cookie da sessão não foi lido, antes da rede.
    CookieDaSessaoIlegivel(#[source] ErroArquivoPrivado),
    /// O cookie da sessão está fora do formato do cabeçalho `Cookie`.
    CookieForaDoFormato,
    /// A URL do Controle não é da origem autorizada: o cookie não foi aberto.
    ControleForaDaOrigem(#[source] FalhaTransporte),
    UnidadeDiferente,
    /// O Controle foi lido, mas a unidade publicada não pôde ser extraída.
    UnidadeDoControleIlegivel(#[source] crate::erro_web::ErroCaixa),
    Menu(crate::processo_criar_menu::ErroMenu),
    Lista(ErroListaDeTipos),
    Cadastro(ErroCadastro),
    Formulario,
    /// A URL do retorno da criação não se lê em pares `chave=valor`.
    ConsultaDoRetornoIlegivel(#[source] crate::consulta_do_controlador::ErroConsultaDoControlador),
    /// O retorno da criação não tem exatamente um `id_procedimento`.
    IdDoRetornoIlegivel(#[source] crate::consulta_do_controlador::ErroChaveDaConsulta),
    TipoSemIdentificador(#[source] crate::consulta_do_controlador::ErroIdDaUrl),
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
        Some(DiagnosticoPosEscrita {
            causa_post,
            releitura_falhou: posterior.as_ref().err().map(|erro| FalhaDaReleitura {
                causa: format!("{erro:?}"),
                http: erro.status_http().cloned(),
            }),
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
