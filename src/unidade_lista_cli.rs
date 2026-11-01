//! Caso de uso da caixa: verifica cobertura antes de publicar ou exportar dados.
use crate::{
    arquivo_caixa, configuracao, dominio, erro_fonte, paginas, paginas_capturadas, saida,
    transporte,
};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

pub(super) fn listar_processos_da_unidade(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    paginas_privadas: &[PathBuf],
) -> ExitCode {
    if let Err(erro) = paginas_capturadas::conferir_caminhos(paginas_privadas) {
        return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem());
    }
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
        Err(falha) => return erro_fonte(json_mode, falha.subcodigo(), falha.mensagem()),
    };
    let paginas_adicionais =
        match paginas_capturadas::ler_complementos(paginas_privadas, html.len()) {
            Ok(paginas) => paginas,
            Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        };
    let mut paginas = Vec::with_capacity(paginas_adicionais.len() + 1);
    paginas.push(html);
    paginas.extend(paginas_adicionais);
    let origem = if paginas_privadas.is_empty() {
        "web"
    } else {
        "web_com_paginas_privadas"
    };
    mostrar_caixa_com_politica(&paginas, origem, unidade, json_mode, true, destino, true)
}

pub(super) fn mostrar_caixa(
    htmls: &[String],
    origem: &str,
    unidade: Option<&str>,
    json_mode: bool,
    exigir_todas_paginas: bool,
    destino: Option<&Path>,
) -> ExitCode {
    mostrar_caixa_com_politica(
        htmls,
        origem,
        unidade,
        json_mode,
        exigir_todas_paginas,
        destino,
        false,
    )
}

/// A consulta Web exige proveniência; replays de capturas podem não trazê-la.
fn conferir_identidade_web(leitura: &paginas::Leitura) -> Result<(), crate::erro_web::ErroCaixa> {
    leitura
        .unidade
        .as_deref()
        .filter(|rotulo| !rotulo.is_empty())
        .map(|_| ())
        .ok_or(crate::erro_web::ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle)
}

fn mostrar_caixa_com_politica(
    htmls: &[String],
    origem: &str,
    unidade: Option<&str>,
    json_mode: bool,
    exigir_todas_paginas: bool,
    destino: Option<&Path>,
    exigir_identidade_web: bool,
) -> ExitCode {
    let leitura = match paginas::juntar(htmls, unidade) {
        Ok(valor) => valor,
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": "tela_inesperada",
                        "subcodigo": erro.subcodigo(),
                        "erro": erro.to_string(),
                        "categoria": erro.categoria().map(dominio::Categoria::rotulo),
                        "id": erro.id(),
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: {erro}");
            }
            return ExitCode::from(1);
        }
    };
    if exigir_identidade_web && let Err(erro) = conferir_identidade_web(&leitura) {
        return erro_fonte(
            json_mode,
            erro.subcodigo(),
            "unidade não identificada no Controle Web",
        );
    }
    // Consulta de sessão promete a caixa inteira; replay offline só relata
    // as páginas selecionadas, sem alegar que representam toda a unidade.
    if exigir_todas_paginas
        && (!leitura.recebidos.completa(&leitura.caixa.recebidos)
            || !leitura.gerados.completa(&leitura.caixa.gerados))
    {
        return erro_fonte(
            json_mode,
            "paginas_do_controle_nao_obtidas",
            "o Controle de Processos possui páginas que não foram obtidas",
        );
    }
    let resultado = saida::json(&leitura, origem);
    if let Some(caminho) = destino {
        match arquivo_caixa::gravar(caminho, &resultado) {
            Ok(bytes) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({"arquivo":"gravado","bytes":bytes,"escopo":"caixa_verificada"})
                    );
                } else {
                    println!("IES: caixa verificada gravada em arquivo privado ({bytes} bytes).");
                }
                return ExitCode::SUCCESS;
            }
            Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        }
    }
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resultado).unwrap());
    } else {
        println!("{}", saida::humano(&leitura));
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod testes_exportacao {
    use super::*;
    use std::fs;

    #[test]
    fn controle_resumido_paginado_nao_exporta_falso_total() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <input id='hdnTipoVisualizacao' value='R'>\
                    <input id='hdnRecebidosHashCriterios' value='filtro-r'>\
                    <input id='hdnGeradosHashCriterios' value='filtro-g'>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>\
                    <a id='lnkInfraProximaPaginaDetalhado'>Próxima</a>";
        let destino = std::env::temp_dir().join(format!(
            "ies-resumida-paginada-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let status = mostrar_caixa_com_politica(
            &[html.to_owned()],
            "web",
            Some("PPI"),
            true,
            true,
            Some(&destino),
            true,
        );
        assert_eq!(status, ExitCode::from(1));
        assert!(!destino.exists());
    }

    #[test]
    fn caixa_observada_com_cobertura_total_pode_ser_exportada() {
        let pasta = std::env::temp_dir().join(format!("ies-caixa-saida-{}", std::process::id()));
        fs::create_dir_all(&pasta).unwrap();
        let arquivo = pasta.join(format!(
            "caixa-{}.json",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                  <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let codigo = mostrar_caixa(
            &[html.to_owned()],
            "offline-teste",
            None,
            true,
            true,
            Some(&arquivo),
        );
        assert_eq!(codigo, ExitCode::SUCCESS);
        let valor: serde_json::Value =
            serde_json::from_slice(&fs::read(&arquivo).unwrap()).unwrap();
        assert_eq!(valor["recebidos"]["total_declarado"], 0);
        assert_eq!(valor["gerados"]["total_declarado"], 0);
        assert!(valor.get("completa").is_none());
        fs::remove_file(arquivo).unwrap();
    }

    #[test]
    fn controle_web_com_cobertura_integral_sem_identidade_de_unidade_falha() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = paginas::juntar(&[html.to_owned()], None).unwrap();
        assert_eq!(
            conferir_identidade_web(&leitura),
            Err(crate::erro_web::ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle)
        );
    }

    #[test]
    fn controle_web_nao_grava_exportacao_se_unidade_nao_foi_identificada() {
        let html = "<table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let caminho = std::env::temp_dir().join(format!(
            "ies-caixa-unidade-ausente-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let codigo = mostrar_caixa_com_politica(
            &[html.to_owned()],
            "web",
            None,
            true,
            true,
            Some(&caminho),
            true,
        );
        assert_eq!(codigo, ExitCode::from(1));
        assert!(!caminho.exists());
    }

    #[test]
    fn unidade_comprovada_passando_pelas_duas_tabelas_eh_aceita() {
        let html = "<a id='lnkInfraUnidade'>PPI</a>\
                    <table id='tblProcessosRecebidos'><caption>0 registros:</caption></table>\
                    <table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let leitura = paginas::juntar(&[html.to_owned()], None).unwrap();
        assert_eq!(leitura.unidade.as_deref(), Some("PPI"));
        assert_eq!(conferir_identidade_web(&leitura), Ok(()));
    }
}
