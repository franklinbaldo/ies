//! Leitor do rótulo da unidade ativa, sem paginação nem mudança de sessão.
use crate::{configuracao, erro_fonte, paginas, transporte};
use crate::erro_web::ErroCaixa;
use serde_json::json;
use std::process::ExitCode;

fn unidade_validada(html: &str) -> Result<String, ErroCaixa> {
    paginas::unidade_exibida_no_controle(html)
}

pub(super) fn mostrar_unidade_atual(unidade_esperada: Option<&str>, json_mode: bool) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let html = match transporte::read_box_from_file(
        &arquivos.url_controle,
        &arquivos.cookie,
        &arquivos.origem,
    ) {
        Ok(html) => html,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let unidade = match unidade_validada(&html) {
        Ok(unidade) => unidade,
        Err(erro) => {
            return erro_fonte(
                json_mode,
                erro.subcodigo(),
                "identificação da unidade não aparece no Controle de Processos",
            );
        }
    };
    if unidade_esperada.is_some_and(|esperada| esperada != unidade) {
        return erro_fonte(
            json_mode,
            "unidade_exibida_diferente_da_solicitada",
            "a unidade mostrada pelo SEI não corresponde à solicitada",
        );
    }
    if json_mode {
        println!(
            "{}",
            json!({"unidade_exibida": unidade, "origem": "controle_de_processos"})
        );
    } else {
        println!("Unidade exibida na sessão SEI: {unidade}");
    }
    ExitCode::SUCCESS
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
