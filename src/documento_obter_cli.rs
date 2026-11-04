//! `ies documento DOC [DOC ...] [--processo NUP] [--saida DIR] [--json]
//! [--unidade SIGLA]` pela Web (Refs #60; contracts/cli/documento-obter-web.md):
//! um documento por vez, na ordem pedida, cada um pelo caminho mais barato
//! (processo informado, índice ou pesquisa rápida), a árvore e o link
//! assinado do conteúdo. Publica metadados e
//! texto visível; nunca URL, hash, cookie ou HTML. Com `--saida`, grava
//! `DOC.txt` (0600, sem sobrescrever) só depois de todos lidos.
use crate::arquivo_texto_documento;
use crate::documento::NumeroDocumento;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{
    DocumentoObtido, ErroDocumentoObter, PedidoDoDocumento, SessaoDoProcesso, obter_documento,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, Domain, configuracao, erro_fonte, saida_erro, transporte};
use serde_json::{Value, json};
use std::path::Path;
use std::process::ExitCode;

/// Metadados de um documento lido, sem o texto.
pub(crate) fn json_do_documento(d: &DocumentoObtido) -> serde_json::Map<String, Value> {
    let valor = json!({
        "numero": d.numero,
        "id_documento": d.id_documento,
        "rotulo": d.rotulo,
        "tipo": d.tipo,
        "icone": d.natureza.icone,
        "cancelado": d.natureza.cancelado,
        "pasta": d.pasta,
        "processo": {"nup": d.processo.nup, "id": d.processo.id, "tipo": d.processo.tipo},
        "unidade": d.unidade,
        "resolvido_por": d.resolvido_por.como_texto(),
        "requisicoes": d.requisicoes,
        "fonte": "documento_visualizar",
        "versao_atual": null,
    });
    match valor {
        Value::Object(mapa) => mapa,
        _ => serde_json::Map::new(),
    }
}

/// Primeira linha humana de um documento.
pub(crate) fn cabecalho(d: &DocumentoObtido) -> String {
    let unidade = d
        .unidade
        .as_deref()
        .map_or_else(String::new, |u| format!(", unidade {u}"));
    format!(
        "IES: documento {} ({}; {}) do processo {}{unidade}; {} requisições.",
        d.numero, d.rotulo, d.tipo, d.processo.nup, d.requisicoes
    )
}

/// `Some` só para a leitura Web: sem `--html-privado`, subcomando nem
/// opções desta operação ainda não implementadas (`--versao`,
/// `--sem-copia`, `--com-propriedades`, `--ocultar-inacessiveis`, que
/// seguem para o stub). Número inválido fica para a validação comum; NUP
/// inválido em `--processo` é entrada inválida (saída 2).
pub(crate) fn tentar_obter(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(
        comando @ Documento {
            acao: None,
            html_privado: None,
            versao: None,
            sem_copia: false,
            ocultar_inacessiveis: false,
            com_propriedades: false,
            ..
        },
    ) = &cli.command
    else {
        return None;
    };
    if comando.numeros.is_empty() {
        return None;
    }
    let numeros = comando
        .numeros
        .iter()
        .map(|n| NumeroDocumento::novo(n).ok())
        .collect::<Option<Vec<_>>>()?;
    for (i, numero) in numeros.iter().enumerate() {
        if numeros[..i].iter().any(|n| n.as_str() == numero.as_str()) {
            let mensagem = "o mesmo número SEI foi pedido mais de uma vez";
            return Some(invalida(cli.json, "numero_documento_repetido", mensagem));
        }
    }
    let processo = match comando.processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(nup)) => Some(nup),
        Some(Err(erro)) => {
            let mensagem = "número de processo inválido";
            return Some(invalida(cli.json, erro.subcodigo(), mensagem));
        }
    };
    Some(obter(
        cli,
        &numeros,
        processo.as_ref(),
        comando.saida.as_deref().map(Path::new),
    ))
}

fn invalida(json_mode: bool, subcodigo: &str, mensagem: &str) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": subcodigo,
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [{subcodigo}]");
    }
    ExitCode::from(2)
}

fn obter(
    cli: &Cli,
    numeros: &[NumeroDocumento],
    processo: Option<&NumeroProcesso>,
    saida: Option<&Path>,
) -> ExitCode {
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    let lidos = transporte::arquivo_privado(&arquivos.url_controle, 8_192).and_then(|url| {
        transporte::arquivo_privado(&arquivos.origem, 512).map(|origem| (url, origem))
    });
    let (url, origem) = match lidos {
        Ok(lidos) => lidos,
        Err(falha) => return erro_fonte(cli.json, falha.subcodigo(), falha.mensagem()),
    };
    let sessao = SessaoDoProcesso {
        url_do_controle: url.trim(),
        origem: origem.trim(),
        cookie_file: &arquivos.cookie,
    };
    let indice = IndiceLocal::da_configuracao(&arquivos);
    let mut obtidos = Vec::new();
    for numero in numeros {
        let pedido = PedidoDoDocumento {
            numero,
            unidade: cli.unidade.as_deref(),
            processo,
        };
        match obter_documento(&sessao, &pedido, &indice, &mut EnvioHttps) {
            Ok(obtido) => obtidos.push(obtido),
            Err(erro) => return falha(cli.json, numero.as_str(), obtidos.len(), &erro),
        }
    }
    publicar(cli.json, &obtidos, saida)
}

/// A falha do documento `numero`; os já lidos não são publicados.
fn falha(json_mode: bool, numero: &str, lidos: usize, erro: &ErroDocumentoObter) -> ExitCode {
    let (subcodigo, mensagem, http) = (erro.subcodigo(), erro.mensagem(), erro.status_http());
    if json_mode {
        let mut valor = saida_erro::json_de_erro_fonte(subcodigo, mensagem, http);
        if let Some(objeto) = valor.as_object_mut() {
            objeto.insert("documento".to_owned(), json!(numero));
            objeto.insert("documentos_lidos_antes".to_owned(), json!(lidos));
        }
        println!("{valor}");
    } else {
        eprintln!(
            "{} (documento {numero})",
            saida_erro::linha_humana(subcodigo, mensagem, http)
        );
    }
    ExitCode::from(1)
}

fn publicar(json_mode: bool, obtidos: &[DocumentoObtido], saida: Option<&Path>) -> ExitCode {
    let mut gravados = Vec::new();
    if let Some(diretorio) = saida {
        for d in obtidos {
            let destino = diretorio.join(format!("{}.txt", d.numero));
            match arquivo_texto_documento::salvar(&destino, &d.texto) {
                Ok(bytes) => gravados.push(bytes),
                Err(erro) => {
                    return erro_fonte(json_mode, erro.subcodigo(), "o texto não foi gravado");
                }
            }
        }
    }
    let total: u32 = obtidos.iter().map(|d| u32::from(d.requisicoes)).sum();
    if json_mode {
        let documentos: Vec<Value> = obtidos
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let mut objeto = json_do_documento(d);
                match gravados.get(i) {
                    Some(bytes) => {
                        objeto.insert("arquivo".to_owned(), json!(format!("{}.txt", d.numero)));
                        objeto.insert("bytes".to_owned(), json!(bytes));
                    }
                    None => {
                        objeto.insert("texto".to_owned(), json!(d.texto));
                    }
                }
                Value::Object(objeto)
            })
            .collect();
        let valor = json!({"documentos": documentos, "requisicoes": total});
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).unwrap_or_default()
        );
    } else {
        for (i, d) in obtidos.iter().enumerate() {
            println!("{}", cabecalho(d));
            match gravados.get(i) {
                Some(bytes) => println!("Texto gravado em {}.txt ({bytes} bytes).", d.numero),
                None => println!("{}", d.texto),
            }
        }
    }
    ExitCode::SUCCESS
}
