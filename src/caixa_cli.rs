//! Consulta do Controle de Processos: apresenta ou grava somente a caixa validada.
use crate::{arquivo_caixa, configuracao, dominio, erro_fonte, paginas, saida, transporte};
use serde_json::json;
use std::process::ExitCode;

pub(super) fn listar_processos_da_unidade(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&std::path::Path>,
) -> ExitCode {
    if let Some(destino) = destino
        && let Err(erro) = arquivo_caixa::validar_destino(destino)
    {
        return erro_fonte(
            json_mode,
            erro.subcodigo(),
            "o destino local da caixa não pode ser utilizado",
        );
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
    mostrar_caixa(&[html], "web", unidade, json_mode, true, destino)
}

pub(super) fn mostrar_caixa(
    htmls: &[String],
    origem: &str,
    unidade: Option<&str>,
    json_mode: bool,
    exigir_todas_paginas: bool,
    destino: Option<&std::path::Path>,
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
    // Consulta de sessão promete a caixa inteira; replay offline só relata
    // as páginas selecionadas, sem alegar que representam toda a unidade.
    if (exigir_todas_paginas || destino.is_some())
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
    if let Some(destino) = destino {
        if let Err(erro) = arquivo_caixa::salvar_exclusivo(destino, &resultado) {
            return erro_fonte(
                json_mode,
                erro.subcodigo(),
                "a caixa não pôde ser gravada no arquivo solicitado",
            );
        }
        if json_mode {
            println!("{}", json!({
                "arquivo": destino.display().to_string(),
                "salvo": true
            }));
        } else {
            println!("IES: caixa validada gravada em {}", destino.display());
        }
        return ExitCode::SUCCESS;
    }
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&resultado).unwrap());
    } else {
        println!("{}", saida::humano(&leitura));
    }
    ExitCode::SUCCESS
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leitura_parcial_nao_gera_arquivo_de_caixa() {
        let pasta = std::env::temp_dir().join(format!(
            "ies-caixa-parcial-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        std::fs::create_dir(&pasta).unwrap();
        let destino = pasta.join("caixa.json");
        let html = "<html><a id='lnkInfraUnidade'>TESTE</a>\
            <table id='tblProcessosRecebidos'><caption>3 registros:</caption>\
            <tr id='P1'><td><a class='processoVisualizado'>001</a></td></tr>\
            </table><table id='tblProcessosGerados'><caption>0 registros:</caption></table>";
        let retorno = mostrar_caixa(
            &[html.to_owned()], "offline", Some("TESTE"), true, false, Some(&destino)
        );
        assert_eq!(retorno, ExitCode::from(1));
        assert!(!destino.exists());
        std::fs::remove_dir_all(pasta).unwrap();
    }
}
