//! `ies documento DOC [DOC ...] [--saida DIR] [--json] [--unidade SIGLA]`
//! pela Web (Refs #60, fatia 1; contracts/cli/documento-obter-web.md): um
//! documento por vez, na ordem pedida, cada um pela pesquisa rápida do
//! número SEI, a árvore e o link assinado do conteúdo. Publica metadados e
//! texto visível; nunca URL, hash, cookie ou HTML. Com `--saida`, grava
//! `DOC.txt` (0600, sem sobrescrever) só depois de todos lidos; o externo
//! (fatia 3) liga o parcial `DOC.pdf`/`.txt`/`.html` gravado na leitura.
use crate::anexo_documento::LimiteDoAnexo;
use crate::arquivo_texto_documento;
use crate::documento::NumeroDocumento;
use crate::indice_local::IndiceLocal;
use crate::processo_obter::{
    DocumentoObtido, ErroDocumentoObter, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso,
    obter_documento,
};
use crate::visao_do_controle::EnvioHttps;
use crate::{Cli, Documento, Domain, configuracao, erro_fonte, saida_erro, transporte};
use serde_json::{Value, json};
use std::path::Path;
use std::process::ExitCode;

#[path = "documento_obter_cli/anexo.rs"]
mod anexo;

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
        "resolvido_por": "pesquisa_rapida",
        "requisicoes": d.requisicoes,
        "fonte": "documento_visualizar",
        "versao_atual": null,
    });
    let mut mapa = match valor {
        Value::Object(mapa) => mapa,
        _ => serde_json::Map::new(),
    };
    if let Some(a) = &d.anexo {
        // Externo: o título do HTML recebido é do órgão de origem (fatia 3).
        mapa.insert("tipo".to_owned(), Value::Null);
        mapa.insert("fonte".to_owned(), json!("documento_download_anexo"));
        mapa.insert("anexo".to_owned(), anexo::json_do_anexo(a));
    }
    mapa
}

/// Primeira linha humana de um documento.
pub(crate) fn cabecalho(d: &DocumentoObtido) -> String {
    let tipo = d.anexo.as_ref().map_or(d.tipo.as_str(), |_| "externo");
    format!(
        "IES: documento {} ({}; {tipo}) do processo {}, unidade {}; {} requisições.",
        d.numero, d.rotulo, d.processo.nup, d.unidade, d.requisicoes
    )
}

/// `Some` só para a leitura Web: sem `--html-privado`, subcomando nem
/// opções desta operação ainda não implementadas (`--processo`, `--versao`,
/// `--sem-copia`, `--com-propriedades`, `--ocultar-inacessiveis`, que
/// seguem para o stub). Número inválido fica para a validação comum.
pub(crate) fn tentar_obter(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(
        comando @ Documento {
            acao: None,
            html_privado: None,
            processo: None,
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
            return Some(repetido(cli.json));
        }
    }
    Some(obter(
        cli,
        &numeros,
        comando.saida.as_deref().map(Path::new),
    ))
}

fn repetido(json_mode: bool) -> ExitCode {
    let mensagem = "o mesmo número SEI foi pedido mais de uma vez";
    if json_mode {
        println!(
            "{}",
            json!({
                "codigo": "entrada_invalida",
                "subcodigo": "numero_documento_repetido",
                "erro": mensagem,
                "retentavel": false
            })
        );
    } else {
        eprintln!("IES: {mensagem} [numero_documento_repetido]");
    }
    ExitCode::from(2)
}

fn obter(cli: &Cli, numeros: &[NumeroDocumento], saida: Option<&Path>) -> ExitCode {
    let Some(limite) = LimiteDoAnexo::do_ambiente() else {
        return anexo::limite_invalido(cli.json);
    };
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
            anexo: PedidoDoAnexo { limite, saida },
        };
        match obter_documento(&sessao, &pedido, &indice, &mut EnvioHttps) {
            Ok(obtido) => obtidos.push(obtido),
            Err(erro) => return falha(cli.json, numero.as_str(), obtidos.len(), &erro),
        }
    }
    publicar(cli.json, obtidos, saida)
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

/// O arquivo gravado de cada documento: nome em `DIR` e bytes.
type Gravado = (String, u64);

/// Grava, na ordem, `DOC.txt` dos internos e liga o parcial `DOC.ext` dos
/// externos; para no primeiro que falha (os anteriores ficam).
fn gravar(
    json_mode: bool,
    obtidos: &mut [DocumentoObtido],
    diretorio: &Path,
) -> Result<Vec<Gravado>, ExitCode> {
    let mut gravados = Vec::new();
    for d in obtidos {
        if let Some(a) = &mut d.anexo {
            let Some(arquivo) = a.arquivo.take() else {
                continue;
            };
            let nome = arquivo.nome().to_owned();
            arquivo
                .concluir()
                .map_err(|erro| erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()))?;
            gravados.push((nome, a.tamanho));
            continue;
        }
        let nome = format!("{}.txt", d.numero);
        match arquivo_texto_documento::salvar(&diretorio.join(&nome), &d.texto) {
            Ok(bytes) => gravados.push((nome, bytes as u64)),
            Err(erro) => {
                return Err(erro_fonte(
                    json_mode,
                    erro.subcodigo(),
                    "o texto não foi gravado",
                ));
            }
        }
    }
    Ok(gravados)
}

fn publicar(json_mode: bool, mut obtidos: Vec<DocumentoObtido>, saida: Option<&Path>) -> ExitCode {
    let mut gravados = Vec::new();
    if let Some(diretorio) = saida {
        match gravar(json_mode, &mut obtidos, diretorio) {
            Ok(lista) => gravados = lista,
            Err(codigo) => return codigo,
        }
    }
    let total: u32 = obtidos.iter().map(|d| u32::from(d.requisicoes)).sum();
    if json_mode {
        let documentos: Vec<Value> = obtidos
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let mut objeto = json_do_documento(d);
                if let Some((nome, bytes)) = gravados.get(i) {
                    objeto.insert("arquivo".to_owned(), json!(nome));
                    objeto.insert("bytes".to_owned(), json!(bytes));
                } else {
                    let texto = match &d.anexo {
                        Some(a) => json!(a.texto),
                        None => json!(d.texto),
                    };
                    objeto.insert("texto".to_owned(), texto);
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
            let gravado = gravados.get(i);
            if let Some(a) = &d.anexo {
                println!("{}", anexo::linha_do_anexo(a, gravado.is_some()));
            }
            match (gravado, &d.anexo) {
                (Some((nome, bytes)), Some(_)) => {
                    println!("Arquivo gravado em {nome} ({bytes} bytes).");
                }
                (Some((nome, bytes)), None) => println!("Texto gravado em {nome} ({bytes} bytes)."),
                (None, Some(a)) => {
                    if let Some(texto) = &a.texto {
                        println!("{texto}");
                    }
                }
                (None, None) => println!("{}", d.texto),
            }
        }
    }
    ExitCode::SUCCESS
}
