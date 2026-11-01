//! Caso de uso de resumo: uma única leitura da caixa, sem abrir processo.
use crate::{
    Cli, Domain, Processo, arquivo_caixa, caixa_local, configuracao, erro_fonte, erro_lista,
    paginas, paginas_capturadas, processo, resumo_exportacao, resumo_pagina, resumo_processo,
    transporte,
};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// Preserva os zeros e normaliza espaços externos antes de comparar o NUP.
fn normalizar_numero_resumo(entrada: &str) -> Result<String, processo::NumeroProcessoInvalido> {
    Ok(processo::NumeroProcesso::novo(entrada)?.as_str().to_owned())
}

/// Reutiliza a junção de caixa: páginas capturadas nunca disparam POST.
fn juntar_paginas_do_resumo(
    primeira: String,
    adicionais: Vec<String>,
    unidade: Option<&str>,
) -> Result<paginas::Leitura, crate::erro_web::ErroCaixa> {
    let mut paginas = Vec::with_capacity(adicionais.len() + 1);
    paginas.push(primeira);
    paginas.extend(adicionais);
    paginas::juntar(&paginas, unidade)
}

fn mostrar_resumo_processo(
    numero: &str,
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    aceitar_pagina_parcial: bool,
    paginas_privadas: &[PathBuf],
) -> ExitCode {
    let preparado = match destino.map(arquivo_caixa::preparar_destino).transpose() {
        Ok(valor) => valor,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    // Complementos locais inviáveis não devem provocar GET administrativo.
    let adicionais = match paginas_capturadas::ler_complementos(paginas_privadas, 0) {
        Ok(paginas) => paginas,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
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
    if let Err(erro) = paginas_capturadas::conferir_orcamento_com_inicial(&adicionais, html.len()) {
        return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem());
    }
    let leitura = match juntar_paginas_do_resumo(html, adicionais, unidade) {
        Ok(leitura) => leitura,
        Err(erro) => {
            return erro_fonte(
                json_mode,
                erro.subcodigo(),
                "Controle de Processos não reconhecido",
            );
        }
    };
    let observacao = match resumo_pagina::observar(&leitura, numero, aceitar_pagina_parcial) {
        Ok(observacao) => observacao,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let escopo = if observacao.pagina_parcial {
        "resumo_da_pagina"
    } else {
        "resumo_da_caixa"
    };
    let mut modelo =
        resumo_exportacao::modelo(numero, leitura.unidade.as_deref(), &observacao.ocorrencias);
    if !paginas_privadas.is_empty() {
        modelo["origem"] = json!("web_com_paginas_privadas");
    }
    if observacao.pagina_parcial {
        modelo["escopo"] = json!(escopo);
        modelo["demais_paginas"] = json!("nao_consultadas");
    }
    if let Some(destino) = preparado.as_ref() {
        return match arquivo_caixa::gravar_preparado(destino, &modelo) {
            Ok(bytes) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "arquivo": "gravado",
                            "bytes": bytes,
                            "escopo": escopo
                        })
                    );
                } else {
                    println!("IES: {escopo} gravado em arquivo privado ({bytes} bytes).");
                }
                ExitCode::SUCCESS
            }
            Err(erro) => erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        };
    }
    if json_mode {
        println!("{modelo}");
    } else if observacao.pagina_parcial {
        println!(
            "IES: processo {numero} observado na página atual ({} ocorrência(s)); demais páginas não consultadas.",
            observacao.ocorrencias.len()
        );
    } else if !paginas_privadas.is_empty() {
        println!(
            "IES: processo {numero} observado na caixa complementada por páginas privadas ({} ocorrência(s)); detalhes não consultados.",
            observacao.ocorrencias.len()
        );
    } else {
        println!(
            "IES: processo {numero} observado na caixa da unidade ({} ocorrência(s)); detalhes não consultados.",
            observacao.ocorrencias.len()
        );
    }
    ExitCode::SUCCESS
}

fn mostrar_resumo_de_caixa_local(
    numero: &str,
    unidade: Option<&str>,
    json_mode: bool,
    caminho: &Path,
) -> ExitCode {
    let observada = match caixa_local::ler(caminho, unidade) {
        Ok(dados) => dados,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let achados = resumo_processo::ocorrencias(&observada.caixa, numero);
    if achados.is_empty() {
        return erro_fonte(
            json_mode,
            "numero_nao_observado_no_arquivo_da_caixa",
            "número não observado no arquivo local; isso não prova inexistência no SEI",
        );
    }
    let mut modelo = resumo_exportacao::modelo(numero, observada.unidade.as_deref(), &achados);
    modelo["fonte"] = json!("arquivo_local");
    modelo["autenticacao_remota"] = json!(false);
    if json_mode {
        println!("{modelo}");
    } else {
        println!(
            "IES: {numero} observado em {} ocorrência(s) no arquivo local; SEI não consultado.",
            achados.len()
        );
    }
    ExitCode::SUCCESS
}

pub(super) fn tentar_resumo(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        numero: Some(numero),
        resumo: true,
        andamentos_completos,
        saida,
        caixa_local,
        observado_na_pagina,
        paginas_capturadas,
        acao: None,
        ..
    }) = &cli.command
    else {
        return None;
    };
    if *andamentos_completos {
        return Some(erro_lista(
            cli.json,
            "opcoes_incompativeis",
            "o resumo da caixa não inclui andamentos",
            2,
        ));
    }
    let normalizado = normalizar_numero_resumo(numero).ok()?;
    let numero = normalizado.as_str();
    if let Some(caminho) = caixa_local.as_deref() {
        return Some(mostrar_resumo_de_caixa_local(
            numero,
            cli.unidade.as_deref(),
            cli.json,
            caminho,
        ));
    }
    Some(mostrar_resumo_processo(
        numero,
        cli.unidade.as_deref(),
        cli.json,
        saida.as_deref(),
        *observado_na_pagina,
        paginas_capturadas,
    ))
}

pub(super) fn validar_numero(cli: &Cli) -> Option<ExitCode> {
    let Domain::Processo(Processo {
        numero: Some(numero),
        acao: None,
        ..
    }) = &cli.command
    else {
        return None;
    };
    let Err(erro) = processo::NumeroProcesso::novo(numero) else {
        return None;
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": erro.subcodigo(),
                "erro": "número de processo inválido",
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: número de processo inválido [{}]", erro.subcodigo());
    }
    Some(ExitCode::from(2))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::erro_web::ErroCaixa;
    use crate::resumo_pagina::FalhaResumoPagina;
    use clap::Parser;

    fn pagina(total: usize, inicio: usize, id: usize, numero: &str) -> String {
        format!(
            "<a id='lnkInfraUnidade'>PPI</a>
             <input id='hdnTipoVisualizacao' value='R'>
             <input id='hdnRecebidosHashCriterios' value='rec'>
             <input id='hdnGeradosHashCriterios' value='ger'>
             <table id='tblProcessosRecebidos'>
               <caption>{total} registros - {inicio} a {inicio}):</caption>
               <tr id='P{id}'><td><a class='processoVisualizado'>{numero}</a></td></tr>
             </table>
             <table id='tblProcessosGerados'><caption>0 registros:</caption></table>"
        )
    }

    #[test]
    fn contrato_de_paginas_requer_resumo_e_recusa_fontes_concorrentes() {
        assert!(
            Cli::try_parse_from([
                "ies",
                "processo",
                "0002/2026",
                "--resumo",
                "--pagina-arquivo",
                "segunda.html"
            ])
            .is_ok()
        );
        for opcoes in [
            vec!["--pagina-arquivo", "segunda.html"],
            vec![
                "--resumo",
                "--pagina-arquivo",
                "segunda.html",
                "--caixa-local",
                "caixa.json",
            ],
            vec![
                "--resumo",
                "--pagina-arquivo",
                "segunda.html",
                "--observado-na-pagina",
            ],
        ] {
            let mut args = vec!["ies", "processo", "0002/2026"];
            args.extend(opcoes);
            assert!(Cli::try_parse_from(args).is_err());
        }
    }

    #[test]
    fn nup_que_so_esta_na_pagina_complementar_requer_cobertura_total() {
        let primeira = pagina(2, 1, 1, "0001/2026");
        let segunda = pagina(2, 2, 2, "0002/2026");
        let parcial = juntar_paginas_do_resumo(primeira.clone(), vec![], Some("PPI")).unwrap();
        assert_eq!(
            resumo_pagina::observar(&parcial, "0002/2026", false),
            Err(FalhaResumoPagina::PaginasDoControleNaoObtidas)
        );
        let consolidada = juntar_paginas_do_resumo(primeira, vec![segunda], Some("PPI")).unwrap();
        let achado = resumo_pagina::observar(&consolidada, "0002/2026", false).unwrap();
        assert!(!achado.pagina_parcial);
        assert_eq!(achado.ocorrencias.len(), 1);
        assert_eq!(consolidada.caixa.recebidos.processos.len(), 2);
    }

    #[test]
    fn paginas_de_outra_unidade_ou_com_intervalo_alterado_sao_recusadas() {
        let primeira = pagina(2, 1, 1, "0001/2026");
        let segunda = pagina(2, 2, 2, "0002/2026");
        let outra = segunda.replace(">PPI</a>", ">OUTRA</a>");
        assert_eq!(
            juntar_paginas_do_resumo(primeira.clone(), vec![outra], Some("PPI"))
                .err()
                .unwrap(),
            ErroCaixa::ContextoDaPaginaDiferenteDoAnterior
        );
        let alterada = pagina(2, 1, 99, "0009/2026");
        assert_eq!(
            juntar_paginas_do_resumo(primeira, vec![alterada], Some("PPI"))
                .err()
                .unwrap(),
            ErroCaixa::FaixaAlterada {
                categoria: crate::dominio::Categoria::Recebidos
            }
        );
    }

    #[test]
    fn resumo_compara_nup_normalizado_sem_perder_zeros_no_inicio() {
        let normalizado = normalizar_numero_resumo("  0001/2026  ").unwrap();
        assert_eq!(normalizado, "0001/2026");
        let leitura =
            juntar_paginas_do_resumo(pagina(1, 1, 1, "0001/2026"), Vec::new(), Some("PPI"))
                .unwrap();
        let observacao = resumo_pagina::observar(&leitura, &normalizado, false).unwrap();
        assert_eq!(observacao.ocorrencias.len(), 1);
        assert_eq!(
            normalizar_numero_resumo("0001/2026\n"),
            Err(processo::NumeroProcessoInvalido::Controle)
        );
    }
}
