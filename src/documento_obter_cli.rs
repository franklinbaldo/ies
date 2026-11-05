//! `ies documento DOC [DOC ...] [--processo NUP] [--saida DIR] [--json]
//! [--unidade SIGLA]` pela Web (Refs #60; contracts/cli/documento-obter-web.md):
//! um documento por vez, na ordem pedida, cada um pelo caminho mais barato
//! (processo informado, índice ou pesquisa rápida), a árvore e o link
//! assinado do conteúdo. Publica metadados e
//! texto visível; nunca URL, hash, cookie ou HTML no terminal. Com
//! `--saida`, grava só a foto `DOC.vN.html` de cada interno (#207,
//! idempotente) depois de todos lidos; o externo (fatia 3) liga o parcial
//! `DOC.pdf`/`.txt`/`.html` gravado na leitura.
//! `--versao N` e a foto: contracts/cli/documento-obter-versoes.md.
use crate::anexo_documento::LimiteDoAnexo;
use crate::documento::NumeroDocumento;
use crate::documento_versao_cli;
use crate::foto_documento::{self, FotoGravada};
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
    let mut versoes = documento_versao_cli::pedido(comando.versao, saida.is_some())?;
    versoes.cache = crate::cache_documento::habilitado_ambiente(comando.sem_cache);
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
            foto_documento::conferir_antes_da_rede(diretorio, numeros, versoes.versao)
    {
        return erro_fonte(cli.json, erro.subcodigo(), "nenhum arquivo foi gravado");
    }
    let arquivos = match configuracao::arquivos_de_acesso() {
        Ok(arquivos) => arquivos,
        Err(erro) => return erro_fonte(cli.json, erro.subcodigo(), erro.mensagem()),
    };
    // Cada tentativa relê a sessão e o índice e recomeça do primeiro DOC.
    let lido = relogin_cli::ler(&arquivos, cli.unidade.as_deref(), || {
        let referencias = transporte::sessao::ReferenciasDaSessao::ler(
            &arquivos.url_controle,
            &arquivos.origem,
            &arquivos.cookie,
        )
        .map_err(FalhaDoDocumento::Sessao)?;
        let sessao = SessaoDoProcesso {
            url_do_controle: referencias.url_do_controle(),
            origem: referencias.origem(),
            cookie: referencias.cookie(),
        };
        let host = crate::instalacao::Instalacao::interpretar(referencias.origem())
            .ok_or(FalhaDoDocumento::Sessao(
                transporte::FalhaTransporte::OrigemNaoAutorizada,
            ))?
            .host()
            .to_owned();
        let indice = IndiceLocal::da_configuracao(&arquivos);
        let mut obtidos = Vec::new();
        for numero in numeros {
            let pedido = PedidoDoDocumento {
                numero,
                unidade: cli.unidade.as_deref(),
                processo,
                anexo: PedidoDoAnexo {
                    limite,
                    saida,
                    cache_host: versoes.cache.then_some(host.as_str()),
                },
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
        Ok((obtidos, host))
    });
    match lido {
        Ok(((obtidos, host), nota)) => publicar(
            cli.json,
            obtidos,
            saida,
            nota.as_ref(),
            &host,
            versoes.cache,
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

/// O arquivo de cada documento em `DIR`: nome, bytes, SHA-256 e se foi
/// gravado agora (`false`: já existia igual).
#[derive(Debug, PartialEq, Eq)]
struct ArquivoDaSaida {
    nome: String,
    bytes: u64,
    sha256: String,
    gravado: bool,
}

impl From<FotoGravada> for ArquivoDaSaida {
    fn from(f: FotoGravada) -> Self {
        Self {
            nome: f.nome,
            bytes: f.bytes as u64,
            sha256: f.sha256,
            gravado: f.gravado,
        }
    }
}

/// Confere as fotos do lote inteiro, liga o parcial `DOC.ext` dos externos
/// e grava as fotos `DOC.vN.html` na ordem.
fn gravar(
    json_mode: bool,
    obtidos: &mut [DocumentoObtido],
    diretorio: &Path,
    caches: &[Option<crate::cache_documento::Registro>],
) -> Result<Vec<Option<ArquivoDaSaida>>, ExitCode> {
    let fotos = if caches.iter().any(Option::is_some) {
        documento_versao_cli::gravar_fotos_com_cache(diretorio, obtidos, caches)
    } else {
        documento_versao_cli::gravar_fotos(diretorio, obtidos)
    }
        .map_err(|erro| erro_fonte(json_mode, erro.subcodigo(), "nenhum arquivo foi gravado"))?;
    let mut gravados = Vec::new();
    for (i, (d, foto)) in obtidos.iter_mut().zip(fotos).enumerate() {
        if let Some(foto) = foto {
            gravados.push(Some(ArquivoDaSaida::from(foto)));
            continue;
        }
        let Some(a) = &mut d.anexo else {
            gravados.push(None);
            continue;
        };
        let Some(arquivo) = a.arquivo.take() else {
            if let Some(Some(cache)) = caches.get(i) {
                let extensao = cache
                    .caminho
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("pdf");
                let nome = format!("{}.{}", d.numero, extensao);
                let destino = diretorio.join(&nome);
                let novo =
                    crate::cache_documento::copiar_para_saida(&cache.caminho, &destino, &a.sha256)
                        .map_err(|erro| erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()))?;
                gravados.push(Some(ArquivoDaSaida {
                    nome,
                    bytes: a.tamanho,
                    sha256: a.sha256.clone(),
                    gravado: novo,
                }));
            } else {
                gravados.push(None);
            }
            continue;
        };
        let nome = arquivo.nome().to_owned();
        arquivo
            .concluir()
            .map_err(|erro| erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()))?;
        gravados.push(Some(ArquivoDaSaida {
            nome,
            bytes: a.tamanho,
            sha256: a.sha256.clone(),
            gravado: true,
        }));
    }
    Ok(gravados)
}

fn publicar(
    json_mode: bool,
    mut obtidos: Vec<DocumentoObtido>,
    destino: Option<&Path>,
    nota: Option<&NotaDoRelogin>,
    host: &str,
    cache_ativo: bool,
) -> ExitCode {
    let mut caches = Vec::new();
    for d in &obtidos {
        let registro = if cache_ativo {
            if let Some(html) = &d.versoes.cache_html {
                match crate::cache_documento::gravar(
                    host,
                    &d.numero,
                    d.versoes
                        .lida
                        .or_else(|| d.versoes.foto.as_ref().map(|f| f.versao))
                        .map(crate::documento_versoes::VersaoDocumento::numero),
                    "html",
                    html.as_bytes(),
                ) {
                    Ok(registro) => Some(registro),
                    Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
                }
            } else if let Some(anexo) = &d.anexo {
                let extensao = match anexo.mime.as_str() {
                    "application/pdf" => "pdf",
                    "text/plain" => "txt",
                    "text/html" => "html",
                    _ => {
                        return erro_fonte(
                            json_mode,
                            "tipo_de_anexo_nao_permitido",
                            "tipo não reconhecido",
                        );
                    }
                };
                let raiz = match crate::cache_documento::raiz() {
                    Ok(raiz) => raiz,
                    Err(erro) => return erro_fonte(json_mode, erro.subcodigo(), erro.mensagem()),
                };
                Some(crate::cache_documento::Registro {
                    caminho: crate::cache_documento::caminho_em(
                        &raiz,
                        host,
                        &d.numero,
                        None,
                        &anexo.sha256,
                        extensao,
                    ),
                    versao: None,
                    sha256: anexo.sha256.clone(),
                })
            } else {
                None
            }
        } else {
            None
        };
        caches.push(registro);
    }
    let mut gravados = Vec::new();
    if let Some(destino) = destino {
        match gravar(json_mode, &mut obtidos, destino, &caches) {
            Ok(lista) => gravados = lista,
            Err(codigo) => return codigo,
        }
    }
    let antes = nota.map_or(0, |n| u32::try_from(n.antes).unwrap_or(u32::MAX));
    let total: u32 = obtidos
        .iter()
        .map(|d| u32::from(d.requisicoes))
        .sum::<u32>()
        .saturating_add(antes);
    let gravado = |i: usize| gravados.get(i).and_then(Option::as_ref);
    if json_mode {
        let documentos: Vec<Value> = obtidos
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let mut objeto = json_do_documento(d);
                documento_versao_cli::inserir_campos(&mut objeto, d);
                objeto.insert(
                    "cache".to_owned(),
                    caches.get(i).and_then(Option::as_ref).map_or(
                        Value::Null,
                        |c| json!({"caminho": c.caminho, "versao": c.versao, "sha256": c.sha256}),
                    ),
                );
                objeto.insert("crc_rodape".to_owned(), json!(d.versoes.crc_rodape));
                if let Some(g) = gravado(i) {
                    let arquivo = documento_versao_cli::json_do_arquivo(
                        &g.nome, g.bytes, &g.sha256, g.gravado,
                    );
                    objeto.insert("arquivo".to_owned(), arquivo);
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
            if let Some(linha) = documento_versao_cli::linha_humana(d) {
                println!("{linha}");
            }
            let g = gravado(i);
            if let Some(a) = &d.anexo {
                println!("{}", anexo::linha_do_anexo(a, g.is_some()));
            }
            match (g, &d.anexo) {
                (Some(g), _) => {
                    let linha = documento_versao_cli::linha_do_arquivo(&g.nome, g.bytes, g.gravado);
                    println!("{linha}");
                }
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
    use crate::documento_versao_cli::testes::{com_foto, diretorio};

    const HTML: &str =
        "<html><head><title>SEI/X - 0011519 - Despacho</title></head><body><p>a</p></body></html>";

    #[test]
    fn saida_grava_so_doc_vn_html_e_repetir_e_no_op() {
        let pasta = diretorio();
        let mut obtidos = vec![com_foto("0011519", 1, HTML)];
        let gravados = gravar(true, &mut obtidos, &pasta, &[]).unwrap();
        let g = gravados[0].as_ref().unwrap();
        assert_eq!((g.nome.as_str(), g.gravado), ("0011519.v1.html", true));
        let de_novo = gravar(true, &mut obtidos, &pasta, &[]).unwrap();
        assert!(!de_novo[0].as_ref().unwrap().gravado);
        let mut nomes: Vec<String> = std::fs::read_dir(&pasta)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        nomes.sort();
        assert_eq!(nomes, ["0011519.v1.html"]);
        std::fs::remove_dir_all(pasta).unwrap();
    }
}
