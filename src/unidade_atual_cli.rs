//! Leitor do rótulo da unidade ativa, sem paginação nem mudança de sessão.
use crate::erro_web::ErroCaixa;
use crate::{configuracao, erro_fonte, relogin_cli, transporte};
use serde_json::json;
use std::process::ExitCode;

pub(crate) fn unidade_validada(html: &str) -> Result<String, ErroCaixa> {
    // Visão resumida ou detalhada: a sessão fica na última visão usada.
    crate::controle_reconhecido::unidade_do_controle(html)
}

fn dados_publicos_unidade(unidade: &str) -> serde_json::Value {
    json!({"unidade_exibida": unidade, "origem": "controle_de_processos"})
}

pub(super) fn mostrar_unidade_atual(unidade_esperada: Option<&str>, json_mode: bool) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.mensagem()),
    };
    // Cada tentativa relê a sessão num retrato coerente (#222).
    let lido = relogin_cli::ler(&arquivos, unidade_esperada, || {
        let sessao = transporte::sessao::ReferenciasDaSessao::ler(
            &arquivos.url_controle,
            &arquivos.origem,
            &arquivos.cookie,
        )?;
        transporte::envio_post::ler_tela_do_controle(&sessao).map(|tela| (sessao, tela))
    });
    let (html, nota) = match lido {
        Ok(((sessao, tela), nota)) => {
            // A colheita usa a geração do cookie que obteve a tela.
            let indice = crate::indice_local::IndiceLocal::da_configuracao(&arquivos);
            crate::colheita_do_controle::Colheita::das_referencias(&indice, &sessao)
                .colher(&tela.html, Some(&tela.mime));
            (tela.html, nota)
        }
        Err(falha) => return relogin_cli::publicar_falha(json_mode, &falha),
    };
    let Ok(unidade) = unidade_validada(&html) else {
        return erro_fonte(
            json_mode,
            "resposta não contém Controle de Processos verificável",
        );
    };
    if unidade_esperada.is_some_and(|esperada| esperada != unidade) {
        return erro_fonte(
            json_mode,
            "a unidade mostrada pelo SEI não corresponde à solicitada",
        );
    }
    if json_mode {
        let mut valor = dados_publicos_unidade(&unidade);
        relogin_cli::anotar_com_total(&mut valor, nota.as_ref());
        println!("{valor}");
    } else {
        relogin_cli::avisar(json_mode, nota.as_ref());
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

#[cfg(test)]
#[path = "unidade_atual_cli/aies_loopback.rs"]
mod aies_loopback;
