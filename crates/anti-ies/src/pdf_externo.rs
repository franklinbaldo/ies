//! Hipótese executável #332: formulários SEI de documento externo e estado
//! em SQLite. Sem cookies, URLs reais, nem rede externa.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::pdf_externo::{self as modelo, Falha, TipoExterno};
use rusqlite::Connection;
use std::fmt::Write as _;

fn html(corpo: String) -> Resposta {
    Resposta {
        estado: Estado::Ok,
        tipo_conteudo: "text/html; charset=utf-8",
        corpo,
    }
}

fn erro(estado: Estado) -> Resposta {
    Resposta::nova(estado)
}

fn falha(causa: &Falha) -> Resposta {
    match causa {
        Falha::Sqlite(_) => erro(Estado::DadosInsuficientes),
        Falha::ProcessoAusente => erro(Estado::NaoEncontrado),
        Falha::TipoAusente
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
    html(format!(
        "<html><body><a id='incluirExterno' href='controlador.php?acao=documento_receber&amp;id_procedimento={id}'>Incluir Documento Externo</a></body></html>"
    ))
}

fn cadastro(id: &str, tipos: &[TipoExterno], selecionado: Option<&str>) -> Resposta {
    let mut pagina = format!(
        "<html><body><form id='frmDocumentoCadastro' method='post'          action='controlador.php?acao=documento_receber&amp;id_procedimento={id}'>         <select id='selSerie' name='selSerie'><option value='null'>Selecione</option>"
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
        // A assinatura inteiramente zerada é sintética; não é infra_hash real.
        let assinatura = "0".repeat(64);
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
    let Ok(Some(processo)) = modelo::processo(banco, origem, cenario, id) else {
        return erro(Estado::NaoEncontrado);
    };
    let Ok(documentos) = modelo::documentos(banco, origem, cenario, id) else {
        return erro(Estado::DadosInsuficientes);
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
            doc.id,
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

fn pdf_do_multipart(corpo: &[u8]) -> Option<(String, u64)> {
    // Hipótese mínima do filArquivo: uma parte multipart com PDF.
    // Não guarda conteúdo no banco; somente nome e comprimento.
    let separador = corpo.windows(4).position(|p| p == b"\r\n\r\n")?;
    let cabecalho = std::str::from_utf8(&corpo[..separador]).ok()?;
    if !cabecalho.contains("name=\"filArquivo\"") {
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
    let restante = &corpo[separador + 4..];
    let fim = restante.windows(4).rposition(|p| p == b"\r\n--")?;
    let pdf = &restante[..fim];
    if !pdf.starts_with(b"%PDF-") {
        return None;
    }
    Some((nome.to_owned(), u64::try_from(pdf.len()).ok()?))
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
            let serie = campo(texto, "selSerie");
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
                                Ok(doc) => html(format!(
                                    "<html><body><a href='controlador.php?acao=arvore_visualizar&amp;id_procedimento={id}&amp;id_documento={}'>Documento {}</a></body></html>",
                                    doc.id,
                                    escapar(&doc.numero_sei)
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
            Some((nome, tamanho)) => {
                match modelo::receber(banco, origem, cenario, id, &nome, tamanho) {
                    Ok(temp) => Resposta {
                        estado: Estado::Ok,
                        tipo_conteudo: "text/plain; charset=utf-8",
                        corpo: format!(
                            "upload-{}#{}#application/pdf#{}#01/01/2026 00:00:00#",
                            temp.id, temp.nome, temp.tamanho
                        ),
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
