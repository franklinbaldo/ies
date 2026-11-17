//! `ies unidade colunas --definir todas|LISTA [--confirmar]`
//! (contracts/cli/unidade-definir-colunas.md): prévia sem escrita; com
//! `--confirmar`, um POST de salvar, conferido por leitura. Erro de uso sai
//! com 2 antes de ler a configuração (sem requisição); efeito incerto, com 3.
//! Vinda da R, a sessão fica na D.
use crate::acao_do_sei::AcaoDoSei;
use crate::configuracao;
use crate::nivel_de_detalhe::{AlvoDasColunas, ColunaDoDetalhe};
use crate::transporte::envio_post::ler_tela_do_controle;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{self, ColunasDefinidas, TelaAtual};
#[cfg(test)]
use serde_json::{Value, json};

#[cfg(test)]
fn chaves(colunas: &[ColunaDoDetalhe]) -> Value {
    json!(colunas.iter().map(|c| c.chave()).collect::<Vec<_>>())
}

/// `alvo`: `--definir` já interpretado pelo clap (uso inválido: saída 2).
pub(crate) fn definir_colunas(
    unidade: Option<&str>,
    json_mode: bool,
    alvo: &AlvoDasColunas,
    confirmar: bool,
) -> anyhow::Result<()> {
    let arquivos = configuracao::arquivos_de_acesso()?;
    let indice = crate::indice_local::IndiceLocal::da_configuracao(&arquivos);
    let sessao =
        ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)?;
    let tela = ler_tela_do_controle(&sessao)?;
    let colheita = crate::colheita_do_controle::Colheita::das_referencias(&indice, &sessao);
    colheita.colher(&tela.html, Some(&tela.mime));
    let atual = TelaAtual {
        url: tela.url,
        html: tela.html,
        mime: tela.mime,
    };
    let feito = visao_do_controle::definir_colunas(
        &tela.instalacao,
        &atual,
        unidade,
        alvo,
        confirmar,
        sessao.cookie(),
        &mut visao_do_controle::EnvioContado::novo(crate::colheita_do_controle::EnvioColhendo {
            colheita: &colheita,
        }),
    )?;
    if json_mode {
        let mut valor = serde_json::to_value(resultado_colunas(&feito))?;
        crate::indice_local::anexar_avisos(&mut valor, &indice);
        println!("{}", serde_json::to_string_pretty(&valor)?);
    } else {
        crate::indice_local::avisar_no_texto(&indice);
        println!("{}", humano(&feito));
    }
    Ok(())
}

/// Contrato do POST: apenas nomes de campos, nunca valores ou hashes.
#[derive(serde::Serialize)]
struct EnvioColunasPublico<'a> {
    acao: AcaoDoSei,
    campos: &'a [String],
    metodo: &'static str,
}

#[derive(serde::Serialize)]
struct ResultadoColunas<'a> {
    alterado: bool,
    alvo: Vec<&'static str>,
    antes: Vec<&'static str>,
    confirmado: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    depois: Option<Vec<&'static str>>,
    desmarcar: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    envio: Option<EnvioColunasPublico<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_salvar: Option<String>,
    indisponiveis: Vec<&'static str>,
    marcar: Vec<&'static str>,
    origem: &'static str,
    reconciliado_por_leitura: bool,
    unidade: &'a str,
}

fn chaves_tipadas(colunas: &[ColunaDoDetalhe]) -> Vec<&'static str> {
    colunas.iter().map(|coluna| coluna.chave()).collect()
}

fn resultado_colunas(feito: &ColunasDefinidas) -> ResultadoColunas<'_> {
    let plano = &feito.plano;
    ResultadoColunas {
        alterado: feito.alterado(),
        alvo: chaves_tipadas(&plano.alvo),
        antes: chaves_tipadas(&plano.antes),
        confirmado: feito.confirmado(),
        depois: feito.depois().map(chaves_tipadas),
        desmarcar: chaves_tipadas(&plano.desmarcar()),
        envio: feito.campos_do_envio().map(|campos| EnvioColunasPublico {
            acao: AcaoDoSei::ProcedimentoConfigurarDetalhe,
            campos,
            metodo: "POST",
        }),
        falha_do_salvar: feito.reconciliado_por().map(|f| format!("{f:?}")),
        indisponiveis: chaves_tipadas(&plano.indisponiveis),
        marcar: chaves_tipadas(&plano.marcar()),
        origem: "web",
        reconciliado_por_leitura: feito.reconciliado_por().is_some(),
        unidade: &feito.unidade,
    }
}

// A montagem antiga por Value permanece somente como oráculo de contrato.
#[cfg(test)]
fn json_de_sucesso(feito: &ColunasDefinidas) -> Value {
    let plano = &feito.plano;
    // Sem valor, a chave é omitida: a saída não traz null.
    let mut objeto = serde_json::Map::new();
    objeto.insert("unidade".to_owned(), json!(feito.unidade));
    objeto.insert("origem".to_owned(), json!("web"));
    objeto.insert("confirmado".to_owned(), json!(feito.confirmado()));
    objeto.insert("alterado".to_owned(), json!(feito.alterado()));
    objeto.insert("antes".to_owned(), chaves(&plano.antes));
    objeto.insert("alvo".to_owned(), chaves(&plano.alvo));
    objeto.insert("marcar".to_owned(), chaves(&plano.marcar()));
    objeto.insert("desmarcar".to_owned(), chaves(&plano.desmarcar()));
    objeto.insert("indisponiveis".to_owned(), chaves(&plano.indisponiveis));
    if let Some(campos) = feito.campos_do_envio() {
        objeto.insert(
            "envio".to_owned(),
            json!({
                "metodo": "POST",
                "acao": AcaoDoSei::ProcedimentoConfigurarDetalhe,
                "campos": campos,
            }),
        );
    }
    if let Some(depois) = feito.depois() {
        objeto.insert("depois".to_owned(), chaves(depois));
    }
    objeto.insert(
        "reconciliado_por_leitura".to_owned(),
        json!(feito.reconciliado_por().is_some()),
    );
    if let Some(falha) = feito.reconciliado_por() {
        objeto.insert("falha_do_salvar".to_owned(), json!(format!("{falha:?}")));
    }
    Value::Object(objeto)
}

fn lista_humana(nomes: &[&str]) -> String {
    if nomes.is_empty() {
        "(nenhuma)".to_owned()
    } else {
        nomes.join(", ")
    }
}

fn linha(rotulo: &str, colunas: &[ColunaDoDetalhe]) -> String {
    let nomes: Vec<_> = colunas.iter().map(|c| c.chave()).collect();
    format!("{rotulo}: {}", lista_humana(&nomes))
}

fn humano(feito: &ColunasDefinidas) -> String {
    let plano = &feito.plano;
    let cabeca = match (
        feito.confirmado(),
        feito.alterado(),
        feito.campos_do_envio(),
    ) {
        (_, _, None) => "nada a salvar: o alvo é a configuração atual",
        (false, _, Some(_)) => "prévia, nada foi salvo",
        (true, true, _) => "salvo e conferido por leitura",
        (true, false, Some(_)) => "não salvo",
    };
    let mut linhas = vec![
        format!(
            "IES: Nível de detalhe da unidade {} — {cabeca}.",
            feito.unidade
        ),
        linha("Atual", &plano.antes),
        linha("Alvo", &plano.alvo),
        linha("Marcar", &plano.marcar()),
        linha("Desmarcar", &plano.desmarcar()),
    ];
    if !plano.indisponiveis.is_empty() {
        linhas.push(linha(
            "Indisponíveis nesta instalação",
            &plano.indisponiveis,
        ));
    }
    if let Some(depois) = feito.depois() {
        linhas.push(linha("Lido depois de salvar", depois));
    }
    if let Some(falha) = feito.reconciliado_por() {
        linhas.push(format!(
            "IES: a resposta do salvar não foi a esperada ({falha:?}); a leitura mostrou o alvo."
        ));
    }
    if let (false, Some(campos)) = (feito.confirmado(), feito.campos_do_envio()) {
        linhas.push(format!("POST que seria enviado: {}", campos.join(", ")));
        linhas.push("Para aplicar, repita com --confirmar.".to_owned());
    }
    linhas.join("\n")
}

#[cfg(test)]
#[path = "unidade_colunas_definir_cli/tests.rs"]
mod tests;
