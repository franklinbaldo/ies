//! Leitor do rótulo da unidade ativa, sem paginação nem mudança de sessão.
use crate::erro_web::ErroCaixa;
use crate::indice_local::{self, IndiceLocal};
use crate::{configuracao, relogin_cli, transporte};
use serde_json::json;

pub(crate) fn unidade_validada(html: &str) -> Result<String, ErroCaixa> {
    // Visão resumida ou detalhada: a sessão fica na última visão usada.
    crate::controle_reconhecido::unidade_do_controle(html)
}

fn dados_publicos_unidade(unidade: &str) -> serde_json::Value {
    json!({"unidade_exibida": unidade, "origem": "controle_de_processos"})
}

/// O Controle mostra outra unidade que a pedida em `--unidade`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) struct UnidadeExibidaDiferenteDaSolicitada;

pub(super) fn mostrar_unidade_atual(
    unidade_esperada: Option<&str>,
    json_mode: bool,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let indice = IndiceLocal::da_configuracao(&arquivos);
    // Cada tentativa relê a sessão num retrato coerente (#222).
    let ((sessao, tela), nota) = relogin_cli::ler(
        &arquivos,
        &indice,
        unidade_esperada,
        crate::transporte::FalhaTransporte::sessao_encerrada,
        || {
            let sessao = transporte::sessao::ReferenciasDaSessao::ler(
                &arquivos.url_controle,
                &arquivos.origem,
                &arquivos.cookie,
            )?;
            transporte::envio_post::ler_tela_do_controle(&sessao).map(|tela| (sessao, tela))
        },
    )
    .map_err(relogin_cli::subir)?;
    // A colheita usa a geração do cookie que obteve a tela.
    crate::colheita_do_controle::Colheita::das_referencias(&indice, &sessao)
        .colher(&tela.html, Some(&tela.mime));
    let unidade = unidade_validada(&tela.html)?;
    if unidade_esperada.is_some_and(|esperada| esperada != unidade) {
        return Err(UnidadeExibidaDiferenteDaSolicitada.into());
    }
    if json_mode {
        let mut valor = dados_publicos_unidade(&unidade);
        relogin_cli::anotar_com_total(&mut valor, nota.as_ref());
        indice_local::anexar_avisos(&mut valor, &indice);
        println!("{valor}");
    } else {
        relogin_cli::avisar(json_mode, nota.as_ref());
        indice_local::avisar_no_texto(&indice);
        println!("Unidade exibida na sessão SEI: {unidade}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dominio::Categoria;

    #[test]
    fn rotulo_isolado_na_tela_de_login_nao_e_controle_verificado() {
        let html =
            "<form id='frmLogin'><input name='txtUsuario'></form><a id='lnkInfraUnidade'>PPI</a>";
        assert_eq!(
            unidade_validada(html),
            Err(ErroCaixa::TabelaAusente {
                categoria: Categoria::Recebidos
            })
        );
    }

    #[test]
    fn uma_tabela_ausente_impede_afirmar_unidade_ativa() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>";
        assert_eq!(
            unidade_validada(html),
            Err(ErroCaixa::TabelaAusente {
                categoria: Categoria::Gerados
            })
        );
    }

    #[test]
    fn controle_com_duas_tabelas_reconhecidas_retorna_unidade_sem_paginar() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        assert_eq!(unidade_validada(html), Ok("PPI".to_owned()));
    }
}

#[cfg(test)]
#[path = "unidade_atual_cli/aies_loopback.rs"]
mod aies_loopback;
