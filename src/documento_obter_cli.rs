//! `ies documento DOC [DOC ...] [--processo NUP] [--saida DIR] [--json]
//! [--unidade SIGLA]` pela Web (Refs #60; contracts/cli/documento-obter-web.md):
//! um documento por vez, na ordem pedida, cada um pelo caminho mais barato
//! (processo informado, índice ou pesquisa rápida), a árvore e o link
//! assinado do conteúdo. Publica metadados e
//! texto visível; nunca URL, hash, cookie ou HTML. Com `--saida`, grava
//! `DOC.txt` (0600, sem sobrescrever) só depois de todos lidos; o externo
//! (fatia 3) liga o parcial `DOC.pdf`/`.txt`/`.html` gravado na leitura.
//! `--versao N` e a cópia de trabalho: contracts/cli/documento-obter-versoes.md (#88).
use crate::anexo_documento::LimiteDoAnexo;
use crate::arquivo_texto_documento;
use crate::documento::NumeroDocumento;
use crate::documento_versao_cli::{self, CopiaGravada};
use crate::documento_versoes::VersaoDocumento;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{
    DocumentoObtido, ErroDocumentoObter, PedidoDeVersoes, PedidoDoAnexo, PedidoDoDocumento,
    SessaoDoProcesso, obter_documento_com_versoes,
};
use crate::relogin_cli::{self, FalhaDaLeitura, NotaDoRelogin};
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
        "resolvido_por": d.resolvido_por.como_texto(),
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
    let unidade = d
        .unidade
        .as_deref()
        .map_or_else(String::new, |u| format!(", unidade {u}"));
    format!(
        "IES: documento {} ({}; {tipo}) do processo {}{unidade}; {} requisições.",
        d.numero, d.rotulo, d.processo.nup, d.requisicoes
    )
}

/// `Some` só para a leitura Web: sem `--html-privado`, subcomando nem
/// opções desta operação ainda não implementadas (`--com-propriedades`,
/// `--ocultar-inacessiveis`, que seguem para o stub). Número inválido e
/// `--versao 0` ficam para a validação comum; NUP inválido em `--processo`
/// é entrada inválida (saída 2).
pub(crate) fn tentar_obter(cli: &Cli) -> Option<ExitCode> {
    let Domain::Documento(
        comando @ Documento {
            acao: None,
            html_privado: None,
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
    let saida = comando.saida.as_deref().map(Path::new);
    let versoes = documento_versao_cli::pedido(comando.versao, saida.is_some(), comando.sem_copia)?;
    let processo = match comando.processo.as_deref().map(NumeroProcesso::novo) {
        None => None,
        Some(Ok(nup)) => Some(nup),
        Some(Err(erro)) => {
            let mensagem = "número de processo inválido";
            return Some(invalida(cli.json, erro.subcodigo(), mensagem));
        }
    };
    Some(obter(cli, &numeros, processo.as_ref(), saida, versoes))
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
    versoes: PedidoDeVersoes,
) -> ExitCode {
    let Some(limite) = LimiteDoAnexo::do_ambiente() else {
        return anexo::limite_invalido(cli.json);
    };
    if let Some(diretorio) = saida
        && let Err(erro) =
            documento_versao_cli::conferir_textos_antes_da_rede(diretorio, numeros, versoes.versao)
    {
        return erro_fonte(cli.json, erro.subcodigo(), "o texto não foi gravado");
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    // Cada tentativa relê a sessão e o índice e recomeça do primeiro DOC.
    let lido = relogin_cli::ler(&arquivos, cli.unidade.as_deref(), || {
        let referencias =
            transporte::sessao::ReferenciasDaSessao::ler(&arquivos.url_controle, &arquivos.origem)
                .map_err(FalhaDoDocumento::Sessao)?;
        let sessao = SessaoDoProcesso {
            url_do_controle: referencias.url_do_controle(),
            origem: referencias.origem(),
            cookie_file: &arquivos.cookie,
        };
        let indice = IndiceLocal::da_configuracao(&arquivos);
        let mut obtidos = Vec::new();
        for numero in numeros {
            let pedido = PedidoDoDocumento {
                numero,
                unidade: cli.unidade.as_deref(),
                processo,
                anexo: PedidoDoAnexo { limite, saida },
            };
            match obter_documento_com_versoes(&sessao, &pedido, versoes, &indice, &mut EnvioHttps) {
                Ok(obtido) => obtidos.push(obtido),
                Err(erro) => {
                    return Err(FalhaDoDocumento::Documento {
                        numero: numero.as_str().to_owned(),
                        lidos: obtidos.len(),
                        erro,
                    });
                }
            }
        }
        Ok((obtidos, referencias.origem().to_owned()))
    });
    match lido {
        Ok(((obtidos, origem), nota)) => publicar(
            cli.json,
            obtidos,
            saida,
            &origem,
            versoes.versao,
            nota.as_ref(),
        ),
        Err(FalhaDaLeitura::Relogin(erro)) => crate::sessao_login_cli::falhar(cli.json, &erro),
        Err(FalhaDaLeitura::Leitura { erro, nota }) => match erro {
            FalhaDoDocumento::Sessao(erro) => {
                relogin_cli::publicar_falha(cli.json, &FalhaDaLeitura::Leitura { erro, nota })
            }
            FalhaDoDocumento::Documento {
                numero,
                lidos,
                erro,
            } => {
                relogin_cli::avisar(cli.json, nota.as_ref());
                falha(cli.json, &numero, lidos, &erro, nota.as_ref())
            }
        },
    }
}

/// A falha de uma tentativa: os arquivos da sessão ou um documento.
enum FalhaDoDocumento {
    Sessao(transporte::FalhaTransporte),
    Documento {
        numero: String,
        lidos: usize,
        erro: ErroDocumentoObter,
    },
}

impl relogin_cli::FalhaPublicavel for FalhaDoDocumento {
    fn subcodigo(&self) -> &'static str {
        match self {
            Self::Sessao(falha) => falha.subcodigo(),
            Self::Documento { erro, .. } => erro.subcodigo(),
        }
    }
    fn mensagem(&self) -> &'static str {
        match self {
            Self::Sessao(falha) => falha.mensagem(),
            Self::Documento { erro, .. } => erro.mensagem(),
        }
    }
    fn status_http(&self) -> Option<&crate::transporte::StatusHttpRecusado> {
        match self {
            Self::Sessao(falha) => falha.status_http(),
            Self::Documento { erro, .. } => erro.status_http(),
        }
    }
}

/// A falha do documento `numero`; os já lidos não são publicados.
fn falha(
    json_mode: bool,
    numero: &str,
    lidos: usize,
    erro: &ErroDocumentoObter,
    nota: Option<&NotaDoRelogin>,
) -> ExitCode {
    let (subcodigo, mensagem, http) = (erro.subcodigo(), erro.mensagem(), erro.status_http());
    if json_mode {
        let mut valor = relogin_cli::json_da_falha(subcodigo, mensagem, http, nota);
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

/// Grava, na ordem, `DOC.txt` (ou `DOC.vN.txt` com `--versao N`, #204) dos
/// internos e liga o parcial `DOC.ext` dos externos; para no primeiro que
/// falha (os anteriores ficam).
fn gravar(
    json_mode: bool,
    obtidos: &mut [DocumentoObtido],
    diretorio: &Path,
    pedida: Option<VersaoDocumento>,
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
        let nome = documento_versao_cli::nome_do_texto(&d.numero, pedida);
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

fn publicar(
    json_mode: bool,
    mut obtidos: Vec<DocumentoObtido>,
    saida: Option<&Path>,
    origem: &str,
    pedida: Option<VersaoDocumento>,
    nota: Option<&NotaDoRelogin>,
) -> ExitCode {
    let mut gravados = Vec::new();
    let mut copias: Vec<Option<CopiaGravada>> = Vec::new();
    if let Some(diretorio) = saida {
        match gravar(json_mode, &mut obtidos, diretorio, pedida) {
            Ok(lista) => gravados = lista,
            Err(codigo) => return codigo,
        }
        for d in &obtidos {
            match documento_versao_cli::gravar_copia(diretorio, origem, d) {
                Ok(copia) => copias.push(copia),
                Err(erro) => {
                    let mensagem = "a cópia de trabalho não foi gravada";
                    return erro_fonte(json_mode, erro.subcodigo(), mensagem);
                }
            }
        }
    }
    let antes = nota.map_or(0, |n| u32::try_from(n.antes).unwrap_or(u32::MAX));
    let total: u32 = obtidos
        .iter()
        .map(|d| u32::from(d.requisicoes))
        .sum::<u32>()
        .saturating_add(antes);
    if json_mode {
        let documentos: Vec<Value> = obtidos
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let mut objeto = json_do_documento(d);
                let copia = copias.get(i).and_then(Option::as_ref);
                documento_versao_cli::inserir_campos(&mut objeto, d, copia);
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
        let mut valor = json!({"documentos": documentos, "requisicoes": total});
        relogin_cli::anotar(&mut valor, nota);
        println!(
            "{}",
            serde_json::to_string_pretty(&valor).unwrap_or_default()
        );
    } else {
        relogin_cli::avisar(json_mode, nota);
        for (i, d) in obtidos.iter().enumerate() {
            println!("{}", cabecalho(d));
            let copia = copias.get(i).and_then(Option::as_ref);
            if let Some(linha) = documento_versao_cli::linha_humana(d, copia) {
                println!("{linha}");
            }
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

#[cfg(test)]
mod testes {
    use super::*;
    use crate::documento_versao_cli::testes::{diretorio, documento, v};
    use crate::processo_obter::{SemCopia, VersoesDoDocumento};

    #[test]
    fn versao_pedida_grava_doc_vn_txt_ao_lado_do_doc_txt() {
        let pasta = diretorio();
        std::fs::write(pasta.join("0011519.txt"), "atual").unwrap();
        let mut obtidos = vec![documento(VersoesDoDocumento {
            lida: Some(v(1)),
            atual: Ok(v(2)),
            copia: Err(SemCopia::VersaoHistorica),
        })];
        let gravados = gravar(true, &mut obtidos, &pasta, Some(v(1))).unwrap();
        assert_eq!(gravados, vec![("0011519.v1.txt".to_owned(), 1)]);
        assert_eq!(
            std::fs::read_to_string(pasta.join("0011519.v1.txt")).unwrap(),
            "a"
        );
        assert_eq!(
            std::fs::read_to_string(pasta.join("0011519.txt")).unwrap(),
            "atual"
        );
        std::fs::remove_dir_all(pasta).unwrap();
    }
}
