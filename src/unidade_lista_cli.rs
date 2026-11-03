//! Caso de uso da caixa: verifica cobertura antes de publicar ou exportar dados.
//! `ies unidade listar-processos` lê a visão detalhada; vinda da R, a sessão
//! fica na D (contracts/cli/unidade-listar-processos-visao-detalhada.md).
use crate::caixa_detalhada::LeituraDetalhada;
use crate::colheita_do_controle::{Colheita, EnvioColhendo};
use crate::indice_local::IndiceLocal;
use crate::transporte::envio_post::ler_tela_do_controle;
use crate::visao_do_controle::{self, ErroVisaoDoControle, TelaAtual};
use crate::{arquivo_caixa, configuracao, dominio, erro_fonte, paginas, saida};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

pub(super) fn listar_processos_da_unidade(
    unidade: Option<&str>,
    json_mode: bool,
    destino: Option<&Path>,
    pagina: Option<usize>,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let colheita = Colheita::da_sessao(&indice, &arquivos);
    let tela =
        match ler_tela_do_controle(&arquivos.url_controle, &arquivos.cookie, &arquivos.origem) {
            Ok(tela) => tela,
            Err(falha) => {
                return crate::saida_erro::erro_fonte_http(
                    json_mode,
                    falha.subcodigo(),
                    falha.mensagem(),
                    falha.status_http(),
                );
            }
        };
    colheita.colher(&tela.html, Some(&tela.mime));
    let atual = TelaAtual {
        url: tela.url,
        html: tela.html,
        mime: tela.mime,
    };
    match visao_do_controle::listar_na_visao_detalhada(
        &tela.instalacao,
        &atual,
        unidade,
        pagina,
        &arquivos.cookie,
        &mut EnvioColhendo {
            colheita: &colheita,
        },
    ) {
        Ok(leitura) => mostrar_caixa_detalhada(&leitura, "web", json_mode, destino),
        Err(erro) => erro_da_visao(json_mode, &erro),
    }
}

/// Falha `fonte_invalida` da troca de visão ou da leitura da D.
pub(crate) fn erro_da_visao(json_mode: bool, erro: &ErroVisaoDoControle) -> ExitCode {
    let valor = crate::saida_erro::json_de_erro_fonte(
        erro.subcodigo(),
        erro.mensagem(),
        erro.status_http(),
    );
    if json_mode {
        println!("{valor}");
    } else {
        eprintln!(
            "{}",
            crate::saida_erro::linha_humana(erro.subcodigo(), erro.mensagem(), erro.status_http())
        );
    }
    ExitCode::from(1)
}

/// A lista D: um array com **uma** lista `categoria: null` e `colunas_ausentes`.
pub(crate) fn json_da_detalhada(leitura: &LeituraDetalhada, origem: &str) -> serde_json::Value {
    let mut lista = saida::lista_json(&saida::ListaObservada {
        unidade: Some(&leitura.unidade),
        origem,
        categoria: None,
        total_declarado: Some(leitura.total_declarado),
        faixa_observada: leitura.faixa_observada,
        processos: &leitura.processos,
    });
    if let Some(objeto) = lista.as_object_mut() {
        objeto.insert(
            "colunas_ausentes".to_owned(),
            json!(leitura.colunas_ausentes()),
        );
    }
    serde_json::Value::Array(vec![lista])
}

/// D é modo de exibição da sessão, não categoria: uma lista, `categoria: null`.
pub(crate) fn mostrar_caixa_detalhada(
    leitura: &LeituraDetalhada,
    origem: &str,
    json_mode: bool,
    destino: Option<&Path>,
) -> ExitCode {
    let saida = json_da_detalhada(leitura, origem);
    if let Some(caminho) = destino {
        return match arquivo_caixa::gravar(caminho, &saida) {
            Ok(bytes) => {
                if json_mode {
                    println!(
                        "{}",
                        json!({
                            "arquivo": "gravado",
                            "bytes": bytes,
                            "escopo": "caixa_detalhada_verificada"
                        })
                    );
                } else {
                    println!("IES: caixa detalhada verificada gravada em arquivo privado.");
                }
                ExitCode::SUCCESS
            }
            Err(erro) => erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
        };
    }
    if json_mode {
        println!("{}", serde_json::to_string_pretty(&saida).unwrap());
    } else {
        println!("{}", humano_da_detalhada(leitura));
    }
    ExitCode::SUCCESS
}

fn humano_da_detalhada(leitura: &LeituraDetalhada) -> String {
    use std::fmt::Write as _;
    let faixa = leitura
        .faixa_observada
        .map_or_else(|| "nenhuma".to_owned(), |(i, f)| format!("{i} a {f}"));
    let mut texto = format!(
        "IES: visão detalhada da unidade {}: {} processo(s) observado(s); total declarado: {}; faixa observada: {faixa}.",
        leitura.unidade,
        leitura.processos.len(),
        leitura.total_declarado
    );
    for processo in &leitura.processos {
        let categoria = processo
            .categoria
            .map_or("categoria não observada", dominio::Categoria::rotulo);
        let _ = write!(
            texto,
            "\n{} ({}; {categoria})",
            processo.numero,
            processo.id.as_str()
        );
    }
    let ausentes = leitura.colunas_ausentes();
    if !ausentes.is_empty() {
        let _ = write!(
            texto,
            "\nIES: colunas não exibidas na visão detalhada: {}. Para vê-las, \
             `ies unidade colunas --definir todas` mostra a prévia de marcar todas as opções \
             desta instalação no Nível de detalhe (aplica com --confirmar); `ies unidade colunas` \
             mostra a configuração atual.",
            ausentes.join(", ")
        );
    }
    texto
}

/// Caixa resumida de um replay: Recebidos e Gerados, como observados.
pub(super) fn mostrar_caixa(
    htmls: &[String],
    origem: &str,
    unidade: Option<&str>,
    json_mode: bool,
    exigir_todas_paginas: bool,
    destino: Option<&Path>,
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
    // Só o replay `--todas-paginas` promete todas as posições; o replay
    // seletivo publica a faixa observada.
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
#[path = "unidade_lista_cli/tests_faixa.rs"]
mod tests_faixa;
