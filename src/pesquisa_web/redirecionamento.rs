//! 302 da pesquisa de protocolo, **interpretado e nunca seguido**.
//! Forma observada nas cassettes `processo-busca-*` (UnB, PEN1, UFRJ, Goiás),
//! POSTs 6 e 10: `Location` relativo para `procedimento_trabalhar`.
//! Contrato: docs/arquitetura/0002-submeter-formulario-observado.md.
use crate::dominio::IdProcedimento;
use crate::formulario_web::Redirecionamento302;
use crate::instalacao::Instalacao;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroRedirecionamentoDaPesquisa {
    LocalizacaoDoRedirecionamentoAusente,
    LocalizacaoDoRedirecionamentoAcimaDoLimite,
    LocalizacaoDoRedirecionamentoNaoEhUrlHttps,
    LocalizacaoDoRedirecionamentoForaDaInstalacao,
    ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento,
    RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar,
    IdProcedimentoAusenteNoRedirecionamento,
    IdProcedimentoRepetidoNoRedirecionamento,
    IdProcedimentoNaoDecimalNoRedirecionamento,
    InfraHashAusenteNoRedirecionamento,
    InfraHashRepetidoNoRedirecionamento,
    InfraHashForaDoFormatoNoRedirecionamento,
    UnidadeAtualAusenteNoRedirecionamento,
    UnidadeAtualRepetidaNoRedirecionamento,
    UnidadeAtualAusenteNaUrlDoEnvio,
    UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio,
}

impl ErroRedirecionamentoDaPesquisa {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::LocalizacaoDoRedirecionamentoAusente => "localizacao_do_redirecionamento_ausente",
            Self::LocalizacaoDoRedirecionamentoAcimaDoLimite => {
                "localizacao_do_redirecionamento_acima_do_limite"
            }
            Self::LocalizacaoDoRedirecionamentoNaoEhUrlHttps => {
                "localizacao_do_redirecionamento_nao_eh_url_https"
            }
            Self::LocalizacaoDoRedirecionamentoForaDaInstalacao => {
                "localizacao_do_redirecionamento_fora_da_instalacao"
            }
            Self::ParametroNaoCanonicoNaLocalizacaoDoRedirecionamento => {
                "parametro_nao_canonico_na_localizacao_do_redirecionamento"
            }
            Self::RedirecionamentoParaAcaoDiferenteDeProcedimentoTrabalhar => {
                "redirecionamento_para_acao_diferente_de_procedimento_trabalhar"
            }
            Self::IdProcedimentoAusenteNoRedirecionamento => {
                "id_procedimento_ausente_no_redirecionamento"
            }
            Self::IdProcedimentoRepetidoNoRedirecionamento => {
                "id_procedimento_repetido_no_redirecionamento"
            }
            Self::IdProcedimentoNaoDecimalNoRedirecionamento => {
                "id_procedimento_nao_decimal_no_redirecionamento"
            }
            Self::InfraHashAusenteNoRedirecionamento => "infra_hash_ausente_no_redirecionamento",
            Self::InfraHashRepetidoNoRedirecionamento => "infra_hash_repetido_no_redirecionamento",
            Self::InfraHashForaDoFormatoNoRedirecionamento => {
                "infra_hash_fora_do_formato_no_redirecionamento"
            }
            Self::UnidadeAtualAusenteNoRedirecionamento => {
                "unidade_atual_ausente_no_redirecionamento"
            }
            Self::UnidadeAtualRepetidaNoRedirecionamento => {
                "unidade_atual_repetida_no_redirecionamento"
            }
            Self::UnidadeAtualAusenteNaUrlDoEnvio => "unidade_atual_ausente_na_url_do_envio",
            Self::UnidadeAtualDoRedirecionamentoDiferenteDaDoEnvio => {
                "unidade_atual_do_redirecionamento_diferente_da_do_envio"
            }
        }
    }
}

/// Extrai o `id_procedimento` do 302 e descarta a URL assinada (consumida).
/// Nunca emite GET ao destino: abrir o processo pode registrar recebimento.
pub(crate) fn interpretar_redirecionamento(
    redirecionamento: Redirecionamento302,
    instalacao: &Instalacao,
) -> Result<IdProcedimento, ErroRedirecionamentoDaPesquisa> {
    let _ = (redirecionamento, instalacao);
    unimplemented!("CONTRACT shim: interpretação do 302")
}

#[cfg(test)]
#[path = "tests_redirecionamento.rs"]
mod tests;
