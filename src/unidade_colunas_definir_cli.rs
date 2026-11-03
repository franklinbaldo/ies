//! `ies unidade colunas --definir todas|LISTA [--confirmar]`
//! (contracts/cli/unidade-definir-colunas.md): prévia sem escrita; com
//! `--confirmar`, um POST de salvar, conferido por leitura. Erro de uso sai
//! com 2 antes de ler a configuração (sem requisição); efeito incerto, com 3.
//! Vinda da R, a sessão fica na D.
use crate::nivel_de_detalhe::{AlvoDasColunas, ColunaDoDetalhe, ErroDaListaDeColunas};
use crate::transporte::envio_post::ler_tela_do_controle;
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
        Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
    };
    let indice = crate::indice_local::IndiceLocal::da_configuracao(&arquivos);
    let colheita = crate::colheita_do_controle::Colheita::da_sessao(&indice, &arquivos);
    let tela =
        match ler_tela_do_controle(&arquivos.url_controle, &arquivos.cookie, &arquivos.origem) {
            Ok(tela) => tela,
            Err(falha) => {
                return saida_erro::erro_fonte_http(
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
    match visao_do_controle::definir_colunas(
        &tela.instalacao,
        &atual,
        unidade,
        &alvo,
        confirmar,
        &arquivos.cookie,
        &mut crate::colheita_do_controle::EnvioColhendo {
            colheita: &colheita,
        },
    ) {
        Ok(feito) => {
            if json_mode {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json_de_sucesso(&feito)).unwrap()
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
        Err(erro) => {
            let valor = json_de_falha(&erro);
            publicar_falha(json_mode, &valor, erro.mensagem(), erro.saida())
        }
    }
}

fn erro_de_uso(json_mode: bool, erro: &ErroDaListaDeColunas) -> ExitCode {
    let mut valor = json!({
        "codigo": "entrada_invalida",
        "subcodigo": erro.subcodigo(),
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
        eprintln!("IES: {} [{}]", erro.mensagem(), erro.subcodigo());
    }
    ExitCode::from(2)
}

fn chaves(colunas: &[ColunaDoDetalhe]) -> Value {
    json!(colunas.iter().map(|c| c.chave()).collect::<Vec<_>>())
}

fn opcional(colunas: Option<&[ColunaDoDetalhe]>) -> Value {
    colunas.map_or(Value::Null, chaves)
}

fn json_da_falha_do_salvar(falha: &FalhaDoSalvar) -> Value {
    let mut valor = Map::new();
    valor.insert("subcodigo".to_owned(), json!(falha.subcodigo()));
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

/// JSON do sucesso (prévia, inalterado ou salvo e conferido).
pub(crate) fn json_de_sucesso(feito: &ColunasDefinidas) -> Value {
    let plano = &feito.plano;
    let envio = feito
        .campos_do_envio
        .as_ref()
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
        "confirmado": feito.confirmado,
        "alterado": feito.alterado,
        "antes": chaves(&plano.antes),
        "alvo": chaves(&plano.alvo),
        "marcar": chaves(&plano.marcar()),
        "desmarcar": chaves(&plano.desmarcar()),
        "indisponiveis": chaves(&plano.indisponiveis),
        "envio": envio,
        "depois": opcional(feito.depois.as_deref()),
        "reconciliado_por_leitura": feito.reconciliado_por.is_some(),
    });
    if let (Some(falha), Some(objeto)) = (&feito.reconciliado_por, valor.as_object_mut()) {
        objeto.insert("falha_do_salvar".to_owned(), json_da_falha_do_salvar(falha));
    }
    valor
}

fn json_da_visao(erro: &ErroVisaoDoControle) -> Value {
    saida_erro::json_de_erro_fonte(erro.subcodigo(), erro.mensagem(), erro.status_http())
}

/// JSON da falha.
pub(crate) fn json_de_falha(erro: &ErroDefinirColunas) -> Value {
    let mut valor = match erro {
        ErroDefinirColunas::Leitura(leitura) => json_da_visao(leitura),
        _ => json!({
            "codigo": erro.codigo(),
            "subcodigo": erro.subcodigo(),
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
        ErroDefinirColunas::EfeitoIncertoNaConfiguracaoDeColunas {
            plano,
            falha_do_salvar,
            verificacao,
        } => {
            objeto.insert("antes".to_owned(), chaves(&plano.antes));
            objeto.insert("alvo".to_owned(), chaves(&plano.alvo));
            inserir_falha(objeto, falha_do_salvar.as_ref());
            let (observadas, lida) = match &**verificacao {
                VerificacaoDasColunas::Marcadas(marcadas) => (chaves(marcadas), Value::Null),
                VerificacaoDasColunas::LeituraFalhou(falha) => {
                    (Value::Null, json!(falha.subcodigo()))
                }
            };
            objeto.insert("observadas".to_owned(), observadas);
            objeto.insert("verificacao".to_owned(), lida);
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

fn publicar_falha(json_mode: bool, valor: &Value, mensagem: &str, saida: u8) -> ExitCode {
    if json_mode {
        println!("{valor}");
    } else {
        let subcodigo = valor["subcodigo"].as_str().unwrap_or_default();
        eprintln!("IES: {mensagem} [{subcodigo}]");
        for campo in ["antes", "alvo", "observadas", "depois"] {
            if let Some(lista) = valor[campo].as_array() {
                let nomes: Vec<_> = lista.iter().filter_map(Value::as_str).collect();
                eprintln!("IES: {campo}: {}", lista_humana(&nomes));
            }
        }
        if let Some(falha) = valor["falha_do_salvar"]["subcodigo"].as_str() {
            eprintln!("IES: resposta do salvar: {falha}");
        }
    }
    ExitCode::from(saida)
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
    let cabeca = match (feito.confirmado, feito.alterado, &feito.campos_do_envio) {
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
    if let Some(depois) = &feito.depois {
        linhas.push(linha("Lido depois de salvar", depois));
    }
    if let Some(falha) = &feito.reconciliado_por {
        linhas.push(format!(
            "IES: a resposta do salvar não foi a esperada ({}); a leitura mostrou o alvo.",
            falha.subcodigo()
        ));
    }
    if let (false, Some(campos)) = (feito.confirmado, &feito.campos_do_envio) {
        linhas.push(format!("POST que seria enviado: {}", campos.join(", ")));
        linhas.push("Para aplicar, repita com --confirmar.".to_owned());
    }
    linhas.join("\n")
}

#[cfg(test)]
#[path = "unidade_colunas_definir_cli/tests.rs"]
mod tests;
