//! Hipótese executável #332: formulários SEI de documento externo e estado
//! em SQLite. Sem cookies, URLs reais, nem rede externa.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo::{self as modelo, Falha, TipoExterno};
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(corpo: String) -> Resposta {
    Resposta::html(corpo)
}

fn erro(estado: Estado) -> Resposta {
    Resposta::nova(estado)
}

fn falha(causa: &Falha) -> Resposta {
    match causa {
        Falha::Sqlite(_) => erro(Estado::DadosInsuficientes),
        Falha::ProcessoAusente => erro(Estado::NaoEncontrado),
        Falha::ProcessoNaoAbertoNaUnidade
        | Falha::TipoAusente
        | Falha::UploadAusente
        | Falha::LimiteExcedido
        | Falha::EntradaInvalida => erro(Estado::RequisicaoInvalida),
    }
}

fn id_processo(alvo: &str) -> Option<&str> {
    parametro_unico(alvo, "id_procedimento")
        .filter(|id| !id.is_empty() && id.bytes().all(|b| b.is_ascii_digit()))
}

fn inicio_da_inclusao(id: &str) -> Resposta {
    let assinatura = "0".repeat(64);
    html(format!(
        "<html><body><a id='incluirExterno' href='controlador.php?acao=documento_receber&amp;id_procedimento={id}&amp;infra_hash={assinatura}'>Incluir Documento Externo</a></body></html>"
    ))
}

fn cadastro(id: &str, tipos: &[TipoExterno], selecionado: Option<&str>) -> Resposta {
    // Hash descartável e fixo: só existe no ambiente simulado local.
    let assinatura = "0".repeat(64);
    let mut pagina = format!(
        "<html><body><form id='frmDocumentoCadastro' method='post'          action='controlador.php?acao=documento_receber&amp;id_procedimento={id}&amp;infra_hash={assinatura}'>         <select id='selSerie' name='selSerie'><option value='null'>Selecione</option>"
    );
    for tipo in tipos {
        let atributo = if selecionado == Some(tipo.id.as_str()) {
            " selected='selected'"
        } else {
            ""
        };
        write!(
            pagina,
            "<option value='{}'{atributo}>{}</option>",
            escapar(&tipo.id),
            escapar(&tipo.nome)
        )
        .expect("formatar HTML");
    }
    let selecionado = selecionado.unwrap_or_default();
    write!(
        pagina,
        "</select><input type='hidden' id='hdnIdSerie' name='hdnIdSerie' value='{}'>         <input type='hidden' id='hdnFlagDocumentoCadastro' name='hdnFlagDocumentoCadastro' value='1'>         <input type='hidden' id='hdnAnexos' name='hdnAnexos' value=''>         <input type='text' name='txtDataElaboracao' value='01/01/2026'>         <select name='selTipoConferencia'><option value='0'>Original</option></select>         <input type='radio' name='rdoFormato' value='N' checked>         <input type='radio' name='rdoNivelAcesso' value='0' checked>         <button type='submit'>Salvar</button></form>         <form id='frmAnexos'><input type='file' id='filArquivo' name='filArquivo'></form>",
        escapar(selecionado)
    )
    .expect("formatar HTML");
    if let Some(tipo) = tipos
        .iter()
        .find(|t| Some(t.id.as_str()) == Some(selecionado))
    {
        let tamanho_mb = tipo.limite_bytes / (1024 * 1024);
        write!(
            pagina,
            "<script type='text/javascript'>\nvar arrExt = [];\n             arrExt[0] = {{nome : \"pdf\", tamanho : {tamanho_mb}}};\n             objTabelaAnexos.adicionar([arr['nome_upload'],arr['nome'],arr['data_hora'],arr['tamanho'],infraFormatarTamanhoBytes(arr['tamanho'])]);\n             objUpload = new infraUpload('frmAnexos','controlador.php?acao=documento_upload_anexo&infra_hash={assinatura}&id_procedimento={id}');\n             </script>"
        )
        .expect("formatar script");
    }
    pagina.push_str("</body></html>");
    html(pagina)
}

fn documento_na_arvore(
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let processo = match modelo::processo(banco, origem, cenario, id) {
        Ok(Some(processo)) => processo,
        Ok(None) => return erro(Estado::NaoEncontrado),
        Err(e) => return falha(&e),
    };
    let documentos = match modelo::documentos(banco, origem, cenario, id) {
        Ok(documentos) => documentos,
        Err(e) => return falha(&e),
    };
    let mut pagina = format!(
        "<html><body><span id='numeroProcesso'>{}</span><span id='unidade'>{}</span><ul id='documentos'>",
        escapar(&processo.nup),
        escapar(&processo.unidade)
    );
    for doc in documentos {
        write!(
            pagina,
            "<li class='documento_pdf' data-id='{}' data-numero-sei='{}'>{} {} ({})</li>",
            doc.id_publico(),
            escapar(&doc.numero_sei),
            escapar(&doc.tipo),
            escapar(&doc.numero_sei),
            escapar(&doc.nome)
        )
        .expect("formatar árvore");
    }
    pagina.push_str("</ul></body></html>");
    html(pagina)
}

/// A escolha interna mantém seus controles, e o mesmo processo oferece
/// a inclusão externa quando há uma série de PDF declarada no SQLite.
pub(crate) fn complementar_escolha(
    mut resposta: Resposta,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Some(id) = id_processo(alvo) else {
        return resposta;
    };
    let tipos = match modelo::tipos(banco, origem, cenario, id) {
        Ok(tipos) => tipos,
        Err(e) => return falha(&e),
    };
    if tipos.is_empty() {
        return resposta;
    }
    if resposta.estado == Estado::NaoEncontrado {
        return inicio_da_inclusao(id);
    }
    if resposta.estado == Estado::Ok {
        let assinatura = "0".repeat(64);
        let link = format!(
            "<a id='incluirExterno' href='controlador.php?acao=documento_receber&amp;id_procedimento={id}&amp;infra_hash={assinatura}'>Incluir Documento Externo</a>"
        );
        if let Some(fim) = resposta.corpo.rfind("</body>") {
            resposta.corpo.insert_str(fim, &link);
        }
    }
    resposta
}

/// Uma única árvore do processo mostra documentos internos e PDFs externos,
/// inclusive quando ambos foram cadastrados no mesmo cenário SQLite.
pub(crate) fn complementar_arvore(
    mut resposta: Resposta,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Some(id) = id_processo(alvo) else {
        return resposta;
    };
    if resposta.estado == Estado::NaoEncontrado {
        return documento_na_arvore(banco, origem, cenario, id);
    }
    if resposta.estado != Estado::Ok {
        return resposta;
    }
    let documentos = match modelo::documentos(banco, origem, cenario, id) {
        Ok(documentos) => documentos,
        Err(e) => return falha(&e),
    };
    let mut itens = String::new();
    for doc in documentos {
        write!(
            itens,
            "<li class='documento_pdf' data-id='{}' data-numero-sei='{}'>{} {} ({})</li>",
            doc.id_publico(),
            escapar(&doc.numero_sei),
            escapar(&doc.tipo),
            escapar(&doc.numero_sei),
            escapar(&doc.nome),
        )
        .expect("formatar árvore");
    }
    let Some(fim) = resposta.corpo.find("</ul>") else {
        return erro(Estado::DadosInsuficientes);
    };
    resposta.corpo.insert_str(fim, &itens);
    resposta
}

fn decode(texto: &str) -> Option<String> {
    let mut valor = Vec::new();
    let mut iter = texto.as_bytes().iter().copied();
    while let Some(byte) = iter.next() {
        match byte {
            b'+' => valor.push(b' '),
            b'%' => {
                let a = (iter.next()? as char).to_digit(16)?;
                let b = (iter.next()? as char).to_digit(16)?;
                valor.push(u8::try_from(a * 16 + b).ok()?);
            }
            c if c.is_ascii() => valor.push(c),
            _ => return None,
        }
    }
    String::from_utf8(valor).ok()
}

fn campo(corpo: &str, chave: &str) -> Option<String> {
    let mut valores = corpo
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(nome, _)| *nome == chave);
    let valor = valores.next()?.1;
    if valores.next().is_some() {
        return None;
    }
    decode(valor)
}

fn pdf_do_multipart(corpo: &[u8]) -> Option<(String, Vec<u8>)> {
    // O formulario pode enviar outros campos antes do arquivo. So um
    // filArquivo e permitido, e somente seu conteudo conta para o tamanho.
    let fim_primeira = corpo.windows(2).position(|p| p == b"\r\n")?;
    let delimitador = corpo.get(..fim_primeira)?;
    if !delimitador.starts_with(b"--") || delimitador.len() < 3 {
        return None;
    }
    let terminador = [b"\r\n".as_slice(), delimitador].concat();
    let mut inicio = fim_primeira + 2;
    let mut arquivo = None;
    loop {
        let fim_cabecalho = inicio
            + corpo
                .get(inicio..)?
                .windows(4)
                .position(|p| p == b"\r\n\r\n")?;
        let cabecalho = std::str::from_utf8(corpo.get(inicio..fim_cabecalho)?).ok()?;
        let inicio_dados = fim_cabecalho + 4;
        let fim_dados = inicio_dados
            + corpo
                .get(inicio_dados..)?
                .windows(terminador.len())
                .position(|p| p == terminador.as_slice())?;
        if cabecalho.contains("name=\"filArquivo\"") {
            if arquivo.is_some() {
                return None;
            }
            let nome = cabecalho.split("filename=\"").nth(1)?.split('"').next()?;
            if nome.is_empty()
                || nome.contains(['\r', '\n'])
                || !std::path::Path::new(nome)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("pdf"))
            {
                return None;
            }
            let pdf = corpo.get(inicio_dados..fim_dados)?;
            if !pdf.starts_with(b"%PDF-") {
                return None;
            }
            arquivo = Some((nome.to_owned(), pdf.to_vec()));
        }
        let depois = fim_dados + terminador.len();
        let marcador = corpo.get(depois..depois + 2)?;
        if marcador == b"--" {
            let restante = corpo.get(depois + 2..)?;
            return if restante.is_empty() || restante == b"\r\n" {
                arquivo
            } else {
                None
            };
        }
        if marcador != b"\r\n" {
            return None;
        }
        inicio = depois + 2;
    }
}

/// GET binário pelo id_anexo realmente ligado ao cadastro definitivo.
/// Sem registro e sem bytes, não fornece arquivo fictício.
pub(crate) fn baixar(
    metodo: &str,
    alvo: &str,
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Resposta {
    if metodo != "GET" {
        return erro(Estado::MetodoNaoPermitido);
    }
    let Some(id_anexo) = parametro_unico(alvo, "id_anexo")
        .filter(|id| id.bytes().all(|b| b.is_ascii_digit()))
    else {
        return erro(Estado::RequisicaoInvalida);
    };
    match modelo::conteudo_do_anexo(banco, origem, cenario, id_anexo) {
        Ok(Some(bytes)) => Resposta::pdf(bytes),
        Ok(None) => erro(Estado::NaoEncontrado),
        Err(Falha::EntradaInvalida) => erro(Estado::RequisicaoInvalida),
        Err(e) => falha(&e),
    }
}

/// Atende o conjunto de rotas externas. `None` delega a outras capacidades.
#[must_use]
pub fn atender(
    metodo: &str,
    alvo: &str,
    corpo: &[u8],
    banco: &Connection,
    origem: &Instalacao,
    cenario: &str,
) -> Option<Resposta> {
    let acao = parametro_unico(alvo, "acao")?;
    if !matches!(
        acao,
        "documento_escolher_tipo"
            | "documento_receber"
            | "documento_upload_anexo"
            | "arvore_visualizar"
    ) {
        return None;
    }
    let Some(id) = id_processo(alvo) else {
        return Some(erro(Estado::RequisicaoInvalida));
    };
    let resposta = match (acao, metodo) {
        ("documento_escolher_tipo", "GET") => {
            if matches!(modelo::processo(banco, origem, cenario, id), Ok(Some(_))) {
                inicio_da_inclusao(id)
            } else {
                erro(Estado::NaoEncontrado)
            }
        }
        ("documento_receber", "GET") => match modelo::tipos(banco, origem, cenario, id) {
            Ok(tipos) if !tipos.is_empty() => cadastro(id, &tipos, None),
            Ok(_) => erro(Estado::NaoEncontrado),
            Err(e) => falha(&e),
        },
        ("documento_receber", "POST") => {
            let Ok(texto) = std::str::from_utf8(corpo) else {
                return Some(erro(Estado::RequisicaoInvalida));
            };
            let flag = campo(texto, "hdnFlagDocumentoCadastro");
            // O cadastro final conserva a série no hidden hdnIdSerie; a
            // recarga por tipo também aceita a seleção visível selSerie.
            let serie = campo(texto, "hdnIdSerie").or_else(|| campo(texto, "selSerie"));
            match (flag.as_deref(), serie.as_deref()) {
                (Some("1"), Some(serie)) => match modelo::tipos(banco, origem, cenario, id) {
                    Ok(tipos) if tipos.iter().any(|t| t.id == serie) => {
                        cadastro(id, &tipos, Some(serie))
                    }
                    Ok(_) => erro(Estado::RequisicaoInvalida),
                    Err(e) => falha(&e),
                },
                (Some("2"), Some(serie)) => {
                    let token = campo(texto, "hdnAnexos")
                        .and_then(|linha| linha.split('±').next().map(str::to_owned))
                        .and_then(|campo| campo.strip_prefix("upload-").map(str::to_owned))
                        .and_then(|id| id.parse::<i64>().ok());
                    match token {
                        Some(token) => {
                            match modelo::cadastrar(banco, origem, cenario, id, serie, token) {
                                Ok(doc) => Resposta::redirecionar(format!(
                                    "controlador.php?acao=procedimento_visualizar&id_procedimento={id}&id_documento={}",
                                    doc.id_publico()
                                )),
                                Err(e) => falha(&e),
                            }
                        }
                        None => erro(Estado::RequisicaoInvalida),
                    }
                }
                _ => erro(Estado::RequisicaoInvalida),
            }
        }
        ("documento_upload_anexo", "POST") => match pdf_do_multipart(corpo) {
            Some((nome, bytes)) => {
                match modelo::receber_conteudo(banco, origem, cenario, id, &nome, &bytes) {
                    Ok(temp) => Resposta {
                        estado: Estado::Ok,
                        tipo_conteudo: "text/plain; charset=utf-8",
                        corpo: format!(
                            "upload-{}#{}#application/pdf#{}#01/01/2026 00:00:00#",
                            temp.id, temp.nome, temp.tamanho
                        ),
                        localizacao: None,
                        corpo_binario: None,
                    },
                    Err(e) => falha(&e),
                }
            }
            None => erro(Estado::RequisicaoInvalida),
        },
        ("arvore_visualizar", "GET") => documento_na_arvore(banco, origem, cenario, id),
        _ => erro(Estado::MetodoNaoPermitido),
    };
    Some(resposta)
}
