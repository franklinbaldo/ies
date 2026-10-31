//! Diagnóstico opt-in do Controle Web: nunca revela processos ou links assinados.
use crate::erro_web::ErroCaixa;
use serde_json::Value;
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FalhaControleWeb {
    ControleNaoReconhecido(ErroCaixa),
    IdentidadeUnidadeNaoObservada,
}
impl FalhaControleWeb {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::ControleNaoReconhecido(_) => "controle_web_nao_reconhecido",
            Self::IdentidadeUnidadeNaoObservada => "identidade_unidade_nao_observada",
        }
    }
    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::ControleNaoReconhecido(_) => {
                "resposta não contém Controle de Processos verificável"
            }
            Self::IdentidadeUnidadeNaoObservada => {
                "identidade da unidade não está publicada na resposta"
            }
        }
    }
}

/// Shim de CONTRATO: uma página HTTP 200 não basta para um resultado positivo.
pub(crate) fn projetar(_html: &str) -> Result<Value, FalhaControleWeb> {
    Err(FalhaControleWeb::IdentidadeUnidadeNaoObservada)
}

/// Shim compilável; GREEN fará um único GET se a opção for solicitada.
pub(crate) fn mostrar(json_mode: bool) -> ExitCode {
    crate::erro_fonte(
        json_mode,
        "controle_web_nao_reconhecido",
        "controle Web ainda não verificado",
    )
}

#[cfg(test)]
mod testes {
    use super::*;
    use clap::Parser;

    #[test]
    fn comando_web_requer_opt_in_e_recusa_mistura_com_validacao_local() {
        assert!(
            crate::Cli::try_parse_from(["ies", "status", "--verificar-controle-web"]).is_ok()
        );
        assert!(
            crate::Cli::try_parse_from([
                "ies", "status", "--verificar-controle-web", "--verificar-sessao-local"
            ]).is_err()
        );
        assert!(
            crate::Cli::try_parse_from([
                "ies", "status", "--verificar-controle-web", "--verificar-arquivos"
            ]).is_err()
        );
    }

    #[test]
    fn pagina_de_controle_identificada_sem_expor_conteudo_da_caixa() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let valor = projetar(html).unwrap();
        assert_eq!(valor["controle_web"], "observado");
        assert_eq!(valor["unidade_exibida"], "PPI");
        assert_eq!(valor["autenticacao_remota"], "nao_auditada");
        assert_eq!(valor["conexao"], "get_realizado");
        assert!(valor.get("processos").is_none());
        assert!(!valor.to_string().contains("infra_hash"));
    }

    #[test]
    fn formulario_de_login_devolvido_com_http_200_nao_prova_sessao() {
        let html = "<html><form id='frmLogin'><input name='txtUsuario'></form></html>";
        assert_eq!(
            projetar(html).unwrap_err().subcodigo(),
            "controle_web_nao_reconhecido"
        );
    }

    #[test]
    fn controles_sem_rotulo_de_unidade_nao_provam_contexto() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert_eq!(
            projetar(html),
            Err(FalhaControleWeb::IdentidadeUnidadeNaoObservada)
        );
    }
}
