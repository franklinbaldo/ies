//! `ies unidade colunas --definir todas|LISTA [--confirmar]`
//! (contracts/cli/unidade-definir-colunas.md): prévia sem escrita; com
//! `--confirmar`, um POST de salvar, conferido por leitura. Erro de uso sai
//! com 2 antes de ler a configuração (sem requisição); efeito incerto, com 3.
//! Vinda da R, a sessão fica na D.
use crate::nivel_de_detalhe::{AlvoDasColunas, ColunaDoDetalhe, ErroDaListaDeColunas};
use crate::transporte::envio_post::ler_tela_do_controle;
use crate::transporte::sessao::ReferenciasDaSessao;
use crate::visao_do_controle::{
    self, ColunasDefinidas, ErroDefinirColunas, ErroVisaoDoControle, FalhaDoSalvar, TelaAtual,
    VerificacaoDasColunas,
};
use crate::{configuracao, erro_fonte, saida_erro};
use serde_json::{Map, Value, json};
use std::process::ExitCode;

pub(crate) fn definir_colunas(
    unidade: Option<&str>,
    json_mode: bool,
    definir: &str,
    confirmar: bool,
) -> ExitCode {
    let alvo = match AlvoDasColunas::interpretar(definir) {
        Ok(alvo) => alvo,
        Err(erro) => return erro_de_uso(json_mode, &erro),
    };
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(json_mode, erro.mensagem()),
    };
    let indice = crate::indice_local::IndiceLocal::da_configuracao(&arquivos);
    let lido = ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem, &arquivos.cookie)
        .and_then(|sessao| ler_tela_do_controle(&sessao).map(|tela| (sessao, tela)));
    let (sessao, tela) = match lido {
        Ok(lido) => lido,
        Err(falha) => {
            return saida_erro::erro_fonte_http(json_mode, falha.mensagem(), falha.status_http());
        }
    };
    let colheita = crate::colheita_do_controle::Colheita::das_referencias(&indice, &sessao);
    colheita.colher(&tela.html, Some(&tela.mime));
    let atual = TelaAtual {
        url: tela.url,
        html: tela.html,
        mime: tela.mime,
    };
    match visao_do_controle::definir_colunas(
        &tela.instalacao,
        &atual,
        unidade,
        &alvo,
        confirmar,
        sessao.cookie(),
        &mut visao_do_controle::EnvioContado::novo(crate::colheita_do_controle::EnvioColhendo {
            colheita: &colheita,
        }),
    ) {
        Ok(feito) => {
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&resultado_colunas(&feito))
                        .expect("JSON serializável")
                );
            } else {
                println!("{}", humano(&feito));
            }
            ExitCode::SUCCESS
        }
        // Nada foi salvo: a mesma falha de `ies unidade colunas`.
        Err(ErroDefinirColunas::Leitura(erro)) => {
            crate::unidade_lista_cli::erro_da_visao(json_mode, &erro)
        }
        Err(erro) => publicar_falha(json_mode, &erro),
    }
}

fn erro_de_uso(json_mode: bool, erro: &ErroDaListaDeColunas) -> ExitCode {
    let mut valor = json!({
        "codigo": "entrada_invalida",
        "erro": erro.mensagem(),
        "retentavel": false,
    });
    let coluna = match erro {
        ErroDaListaDeColunas::ColunaDesconhecida { chave } => Some(chave.as_str()),
        ErroDaListaDeColunas::ColunaRepetidaNaLista { coluna } => Some(coluna.chave()),
        _ => None,
    };
    if let (Some(coluna), Some(objeto)) = (coluna, valor.as_object_mut()) {
        objeto.insert("coluna".to_owned(), json!(coluna));
    }
    if json_mode {
        println!("{valor}");
    } else {
        eprintln!("IES: {} ({})", erro.mensagem(), erro.mensagem());
    }
    ExitCode::from(2)
}

fn chaves(colunas: &[ColunaDoDetalhe]) -> Value {
    json!(colunas.iter().map(|c| c.chave()).collect::<Vec<_>>())
}

#[cfg(test)]
fn opcional(colunas: Option<&[ColunaDoDetalhe]>) -> Value {
    colunas.map_or(Value::Null, chaves)
}

fn json_da_falha_do_salvar(falha: &FalhaDoSalvar) -> Value {
    let mut valor = Map::new();
    if let Some(http) = falha.status_http() {
        // 200: validação da Infra, só a mensagem.
        if http.status != 200 {
            valor.insert("status_http".to_owned(), json!(http.status));
        }
        if let Some(texto) = http.mensagem_do_servidor {
            valor.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    Value::Object(valor)
}

/// Contrato do POST: apenas nomes de campos, nunca valores ou hashes.
#[derive(serde::Serialize)]
struct EnvioColunasPublico<'a> {
    acao: &'static str,
    campos: &'a [String],
    metodo: &'static str,
}

#[derive(serde::Serialize)]
struct ResultadoColunas<'a> {
    alterado: bool,
    alvo: Vec<&'static str>,
    antes: Vec<&'static str>,
    confirmado: bool,
    depois: Option<Vec<&'static str>>,
    desmarcar: Vec<&'static str>,
    envio: Option<EnvioColunasPublico<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    falha_do_salvar: Option<crate::saida_erro::FalhaPublica>,
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
        envio: feito.campos_do_envio()
            .map(|campos| EnvioColunasPublico {
                acao: crate::nivel_de_detalhe::ACAO_DO_NIVEL_DE_DETALHE,
                campos,
                metodo: "POST",
            }),
        falha_do_salvar: feito.reconciliado_por()
            .map(|falha| crate::saida_erro::FalhaPublica::nova(falha.status_http())),
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
    let envio = feito.campos_do_envio()
        .map_or(Value::Null, |campos| {
            json!({
                "metodo": "POST",
                "acao": crate::nivel_de_detalhe::ACAO_DO_NIVEL_DE_DETALHE,
                "campos": campos,
            })
        });
    let mut valor = json!({
        "unidade": feito.unidade,
        "origem": "web",
        "confirmado": feito.confirmado(),
        "alterado": feito.alterado(),
        "antes": chaves(&plano.antes),
        "alvo": chaves(&plano.alvo),
        "marcar": chaves(&plano.marcar()),
        "desmarcar": chaves(&plano.desmarcar()),
        "indisponiveis": chaves(&plano.indisponiveis),
        "envio": envio,
        "depois": opcional(feito.depois()),
        "reconciliado_por_leitura": feito.reconciliado_por().is_some(),
    });
    if let (Some(falha), Some(objeto)) = (feito.reconciliado_por(), valor.as_object_mut()) {
        objeto.insert("falha_do_salvar".to_owned(), json_da_falha_do_salvar(falha));
    }
    valor
}

fn json_da_visao(erro: &ErroVisaoDoControle) -> Value {
    saida_erro::json_de_erro_fonte(erro.mensagem(), erro.status_http())
}

/// JSON da falha.
pub(crate) fn json_de_falha(erro: &ErroDefinirColunas) -> Value {
    let mut valor = match erro {
        ErroDefinirColunas::Leitura(leitura) => json_da_visao(leitura),
        _ => json!({
            "codigo": erro.codigo(),
            "erro": erro.mensagem(),
            "retentavel": false,
        }),
    };
    let Some(objeto) = valor.as_object_mut() else {
        return valor;
    };
    match erro {
        ErroDefinirColunas::Leitura(_) => {}
        ErroDefinirColunas::ColunaIndisponivelNestaInstalacao { coluna } => {
            objeto.insert("coluna".to_owned(), json!(coluna.chave()));
        }
        ErroDefinirColunas::ConfiguracaoDeColunasNaoAplicada {
            plano,
            falha_do_salvar,
        } => {
            objeto.insert("antes".to_owned(), chaves(&plano.antes));
            objeto.insert("alvo".to_owned(), chaves(&plano.alvo));
            inserir_falha(objeto, falha_do_salvar.as_ref());
        }
        ErroDefinirColunas::MarcasDiferentesDoAlvoAposSalvar { plano, observadas } => {
            objeto.insert("antes".to_owned(), chaves(&plano.antes));
            objeto.insert("alvo".to_owned(), chaves(&plano.alvo));
            objeto.insert("observadas".to_owned(), chaves(observadas));
        }
        ErroDefinirColunas::NaoConfirmadoNaConfiguracaoDeColunas {
            plano,
            falha_do_salvar,
            verificacao,
        } => {
            objeto.insert("antes".to_owned(), chaves(&plano.antes));
            objeto.insert("alvo".to_owned(), chaves(&plano.alvo));
            inserir_falha(objeto, falha_do_salvar.as_ref());
            if let VerificacaoDasColunas::Marcadas(marcadas) = &**verificacao {
                objeto.insert("observadas".to_owned(), chaves(marcadas));
            }
        }
    }
    valor
}

/// A falha do salvar e, no topo, o status e a mensagem do SEI.
fn inserir_falha(objeto: &mut Map<String, Value>, falha: Option<&FalhaDoSalvar>) {
    let Some(falha) = falha else {
        return;
    };
    if let Some(http) = falha.status_http() {
        objeto.insert("status_http".to_owned(), json!(http.status));
        if let Some(texto) = http.mensagem_do_servidor {
            objeto.insert("mensagem_do_servidor".to_owned(), json!(texto));
        }
    }
    objeto.insert("falha_do_salvar".to_owned(), json_da_falha_do_salvar(falha));
}

/// A apresentação humana não depende da projeção JSON.
fn linhas_da_falha(erro: &ErroDefinirColunas) -> Vec<String> {
    use ErroDefinirColunas as E;
    let mut linhas = vec![format!("IES: {}", erro.mensagem())];

    let plano = match erro {
        E::ConfiguracaoDeColunasNaoAplicada { plano, .. }
        | E::MarcasDiferentesDoAlvoAposSalvar { plano, .. }
        | E::NaoConfirmadoNaConfiguracaoDeColunas { plano, .. } => Some(plano.as_ref()),
        _ => None,
    };
    if let Some(plano) = plano {
        linhas.push(format!("IES: {}", linha("antes", &plano.antes)));
        linhas.push(format!("IES: {}", linha("alvo", &plano.alvo)));
    }

    match erro {
        E::MarcasDiferentesDoAlvoAposSalvar { observadas, .. } => {
            linhas.push(format!("IES: {}", linha("observadas", observadas)));
        }
        E::NaoConfirmadoNaConfiguracaoDeColunas { verificacao, .. } => {
            if let VerificacaoDasColunas::Marcadas(observadas) = verificacao.as_ref() {
                linhas.push(format!("IES: {}", linha("observadas", observadas)));
            }
        }
        _ => {}
    }

    let falha = match erro {
        E::ConfiguracaoDeColunasNaoAplicada {
            falha_do_salvar, ..
        }
        | E::NaoConfirmadoNaConfiguracaoDeColunas {
            falha_do_salvar, ..
        } => falha_do_salvar.as_ref(),
        _ => None,
    };
    if let Some(falha) = falha {
        linhas.push(format!("IES: resposta do salvar: {falha:?}"));
    }
    linhas
}

fn publicar_falha(json_mode: bool, erro: &ErroDefinirColunas) -> ExitCode {
    if json_mode {
        println!("{}", json_de_falha(erro));
    } else {
        for linha in linhas_da_falha(erro) {
            eprintln!("{linha}");
        }
    }
    ExitCode::from(1)
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
    let cabeca = match (feito.confirmado(), feito.alterado(), feito.campos_do_envio()) {
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
