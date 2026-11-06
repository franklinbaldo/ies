//! Erros tipados do fluxo de criação; o envio e a releitura preservam as causas.
use super::reconciliacao;
use crate::erro_html::ErroHtml;
use crate::formulario_web::ErroSubmissaoFormulario;
use crate::processo_criar_cadastro::ErroCadastro;
use crate::processo_criar_hipotese::ErroHipotese;
use crate::processo_criar_interessados::ErroInteressado;
use crate::processo_criar_lista::ErroListaDeTipos;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

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
    /// O POST foi tentado; uma leitura única do Controle apurou apenas
    /// candidatos, nunca causalidade. As duas falhas são preservadas.
    PosEscrita {
        causa_post: ErroSubmissaoFormulario,
        posterior: Result<Vec<reconciliacao::CandidatoGerado>, Box<ErroCriar>>,
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
            Self::PosEscrita { .. } | Self::EfeitoIncerto => "efeito_incerto_na_criacao",
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

    pub(crate) fn saida(&self) -> u8 {
        if matches!(self, Self::EfeitoIncerto | Self::PosEscrita { .. }) {
            3
        } else {
            1
        }
    }

    /// O retorno não infere ausência quando não houve candidato novo.
    /// Só expõe subcódigos tipados e NUPs opacos, nunca URLs ou cookies.
    pub(crate) fn diagnostico_apos_post(
        &self,
    ) -> Option<(&'static str, Option<&'static str>, Option<Vec<String>>)> {
        let Self::PosEscrita { causa_post, posterior } = self else {
            return None;
        };
        Some((
            causa_post.subcodigo(),
            posterior.as_ref().err().map(|erro| erro.subcodigo()),
            posterior.as_ref().ok().map(|candidatos| {
                candidatos.iter().map(|c| c.nup.clone()).collect()
            }),
        ))
    }
}

