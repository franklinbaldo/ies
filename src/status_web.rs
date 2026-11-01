//! Diagnóstico opt-in do Controle Web: nunca revela processos ou links assinados.
use crate::erro_web::ErroCaixa;
use crate::{configuracao, paginas, transporte};
use serde_json::{Value, json};
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FalhaControleWeb {
    ControleNaoReconhecido(ErroCaixa),
    IdentidadeUnidadeNaoObservada,
    UnidadeExibidaDiferenteDaSolicitada,
}
impl FalhaControleWeb {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::ControleNaoReconhecido(_) => "controle_web_nao_reconhecido",
            Self::IdentidadeUnidadeNaoObservada => "identidade_unidade_nao_observada",
            Self::UnidadeExibidaDiferenteDaSolicitada => {
                "unidade_exibida_diferente_da_solicitada"
            }
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
            Self::UnidadeExibidaDiferenteDaSolicitada => {
                "a unidade exibida pelo SEI não corresponde à solicitada"
            }
        }
    }
}

/// A tela precisa conter Controle de Processos e unidade única, não apenas HTTP 200.
pub(crate) fn projetar(html: &str) -> Result<Value, FalhaControleWeb> {
    let leitura = paginas::juntar(&[html.to_owned()], None)
        .map_err(FalhaControleWeb::ControleNaoReconhecido)?;
    let unidade = leitura
        .unidade
        .ok_or(FalhaControleWeb::IdentidadeUnidadeNaoObservada)?;
    Ok(json!({
        "controle_web": "observado",
        "unidade_exibida": unidade,
        "conexao": "get_realizado",
        "autenticacao_remota": "nao_auditada",
    }))
}

/// Shim de conferência: o comportamento será guiado por testes do contrato.
fn projetar_com_unidade(
    html: &str,
    unidade_esperada: Option<&str>,
) -> Result<Value, FalhaControleWeb> {
    let _ = unidade_esperada;
    projetar(html)
}

/// Somente com pedido explícito: um GET da sessão já configurada, sem novo login.
pub(crate) fn mostrar(json_mode: bool, unidade_esperada: Option<&str>) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return crate::erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let html = match transporte::read_box_from_file(
        &arquivos.url_controle,
        &arquivos.cookie,
        &arquivos.origem,
    ) {
        Ok(html) => html,
        Err(erro) => return crate::erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let modelo = match projetar_com_unidade(&html, unidade_esperada) {
        Ok(modelo) => modelo,
        Err(erro) => {
            return crate::erro_fonte(json_mode, erro.subcodigo(), erro.mensagem());
        }
    };
    if json_mode {
        println!("{modelo}");
    } else {
        println!(
            "IES: Controle Web observado na unidade {}; autenticação não auditada.",
            modelo["unidade_exibida"].as_str().unwrap_or("desconhecida")
        );
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod testes {
    use super::*;
    use clap::Parser;

    #[test]
    fn comando_web_requer_opt_in_e_recusa_mistura_com_validacao_local() {
        assert!(crate::Cli::try_parse_from(["ies", "status", "--verificar-controle-web"]).is_ok());
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "status",
                "--verificar-controle-web",
                "--verificar-sessao-local"
            ])
            .is_err()
        );
        assert!(
            crate::Cli::try_parse_from([
                "ies",
                "status",
                "--verificar-controle-web",
                "--verificar-arquivos"
            ])
            .is_err()
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
