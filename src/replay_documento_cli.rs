//! Seleção por número exato a partir de HTML de documento já verificado, sem rede.
use crate::documento::{NumeroDocumento, NumeroDocumentoInvalido};
use crate::documento_web::{
    ErroCorpoDocumento, ErroTituloDocumento, IdentidadeDoDocumento, identidade_do_titulo,
    texto_do_corpo,
};
use crate::{arquivo_texto_documento, cassette};
use serde_json::json;
use std::collections::HashSet;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroReplayDocumento {
    FonteNaoVerificada,
    Numero(NumeroDocumentoInvalido),
    Titulo(ErroTituloDocumento),
    Corpo(ErroCorpoDocumento),
    NumeroNaoObservado,
    NumeroRepetido,
}

impl ErroReplayDocumento {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::FonteNaoVerificada => "replay_documento_nao_selecionado",
            Self::Numero(erro) => erro.subcodigo(),
            Self::Titulo(erro) => erro.subcodigo(),
            Self::Corpo(erro) => erro.subcodigo(),
            Self::NumeroNaoObservado => "numero_documento_nao_observado_na_cassette",
            Self::NumeroRepetido => "numero_documento_repetido_na_cassette",
        }
    }

    fn categoria(self) -> (&'static str, u8) {
        if matches!(self, Self::Numero(_)) {
            ("entrada_invalida", 2)
        } else {
            ("fonte_invalida", 1)
        }
    }
}

fn selecionar(
    htmls: &[String],
    numero: &NumeroDocumento,
) -> Result<(IdentidadeDoDocumento, usize), ErroReplayDocumento> {
    let mut encontrado = None;
    for (indice, html) in htmls.iter().enumerate() {
        let documento = identidade_do_titulo(html).map_err(ErroReplayDocumento::Titulo)?;
        if documento.numero == numero.as_str() {
            if encontrado.is_some() {
                return Err(ErroReplayDocumento::NumeroRepetido);
            }
            encontrado = Some((documento, indice));
        }
    }
    encontrado.ok_or(ErroReplayDocumento::NumeroNaoObservado)
}

fn ler(
    card: &Path,
    arquivo: &Path,
    passos: &[usize],
    numero: Option<&str>,
    texto: bool,
) -> Result<(IdentidadeDoDocumento, String, Option<String>), ErroReplayDocumento> {
    if let Some(numero) = numero {
        let solicitado = NumeroDocumento::novo(numero).map_err(ErroReplayDocumento::Numero)?;
        let (paginas, instalacao) = cassette::read_cassette_documents(card, arquivo, passos)
            .map_err(|_| ErroReplayDocumento::FonteNaoVerificada)?;
        let (documento, indice) = selecionar(&paginas, &solicitado)?;
        let conteudo = texto
            .then(|| texto_do_corpo(&paginas[indice]))
            .transpose()
            .map_err(ErroReplayDocumento::Corpo)?;
        Ok((documento, instalacao, conteudo))
    } else {
        let (html, instalacao) = cassette::read_cassette_document(card, arquivo, passos)
            .map_err(|_| ErroReplayDocumento::FonteNaoVerificada)?;
        let documento = identidade_do_titulo(&html).map_err(ErroReplayDocumento::Titulo)?;
        let conteudo = texto
            .then(|| texto_do_corpo(&html))
            .transpose()
            .map_err(ErroReplayDocumento::Corpo)?;
        Ok((documento, instalacao, conteudo))
    }
}

pub(super) struct SaidaDocumento<'a> {
    pub texto: bool,
    pub arquivo: Option<&'a Path>,
}

pub(super) fn mostrar(
    card: &Path,
    arquivo: &Path,
    passos: &[usize],
    numero: Option<&str>,
    listar: bool,
    saida: &SaidaDocumento<'_>,
    json_mode: bool,
) -> ExitCode {
    if listar {
        return mostrar_lista(card, arquivo, passos, json_mode);
    }
    match ler(card, arquivo, passos, numero, saida.texto) {
        Ok((documento, instalacao, conteudo)) => {
            if let Some(destino) = saida.arquivo {
                let Some(texto) = conteudo.as_deref() else {
                    return erro_exportacao("texto_documento_nao_selecionado", json_mode);
                };
                return match arquivo_texto_documento::salvar(destino, texto) {
                    Ok(bytes) => {
                        if json_mode {
                            println!(
                                "{}",
                                json!({"documento":documento,"instalacao":instalacao,
                                "escopo":"texto_documento_exportado_offline","bytes":bytes})
                            );
                        } else {
                            println!(
                                "IES: texto de documento {} gravado em arquivo privado.",
                                documento.numero
                            );
                        }
                        ExitCode::SUCCESS
                    }
                    Err(erro) => erro_exportacao(erro.subcodigo(), json_mode),
                };
            }
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "documento": documento,
                        "texto": conteudo,
                        "instalacao": instalacao,
                        "escopo": if saida.texto { "texto_documento_offline" } else { "titulo_documento_offline" }
                    })
                );
            } else {
                println!(
                    "Documento SEI {}: {} (HTML offline)",
                    documento.numero, documento.tipo
                );
                if let Some(conteudo) = conteudo {
                    println!("{conteudo}");
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            let (codigo, saida) = erro.categoria();
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "codigo": codigo,
                        "subcodigo": erro.subcodigo(),
                        "erro": "seleção offline de documento não concluída",
                        "retentavel": false
                    })
                );
            } else {
                eprintln!("IES: seleção offline não concluída [{}]", erro.subcodigo());
            }
            ExitCode::from(saida)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifica_duplicidade_pelo_tipo_e_nao_por_mensagem() {
        let html = "<title>SEI/UnB - 0007 - Ofício</title>".to_owned();
        let numero = NumeroDocumento::novo("0007").unwrap();
        assert_eq!(
            selecionar(&[html.clone(), html], &numero),
            Err(ErroReplayDocumento::NumeroRepetido)
        );
    }

    #[test]
    fn falta_na_cassette_nao_significa_que_documento_inexiste() {
        let html = "<title>SEI/UnB - 0007 - Ofício</title>".to_owned();
        let numero = NumeroDocumento::novo("7").unwrap();
        assert_eq!(
            selecionar(&[html], &numero),
            Err(ErroReplayDocumento::NumeroNaoObservado)
        );
    }
}

/// Ordem da captura preservada; número e tipo derivam apenas do HTML verificado.
fn listar_identidades(htmls: &[String]) -> Result<Vec<IdentidadeDoDocumento>, ErroReplayDocumento> {
    if htmls.is_empty() {
        return Err(ErroReplayDocumento::FonteNaoVerificada);
    }
    let mut vistos = HashSet::new();
    let mut resultado = Vec::with_capacity(htmls.len());
    for html in htmls {
        let identidade = identidade_do_titulo(html).map_err(ErroReplayDocumento::Titulo)?;
        if !vistos.insert(identidade.numero.clone()) {
            return Err(ErroReplayDocumento::NumeroRepetido);
        }
        resultado.push(identidade);
    }
    Ok(resultado)
}

fn mostrar_lista(card: &Path, arquivo: &Path, passos: &[usize], json_mode: bool) -> ExitCode {
    let resultado = cassette::read_cassette_documents(card, arquivo, passos)
        .map_err(|_| ErroReplayDocumento::FonteNaoVerificada)
        .and_then(|(htmls, instalacao)| {
            listar_identidades(&htmls).map(|documentos| (documentos, instalacao))
        });
    match resultado {
        Ok((documentos, instalacao)) => {
            if json_mode {
                println!(
                    "{}",
                    json!({"documentos":documentos,"instalacao":instalacao,"escopo":"titulos_documentos_offline"})
                );
            } else {
                for item in documentos {
                    println!(
                        "Documento SEI {}: {} (HTML offline)",
                        item.numero, item.tipo
                    );
                }
            }
            ExitCode::SUCCESS
        }
        Err(erro) => {
            if json_mode {
                println!(
                    "{}",
                    json!({"codigo":"fonte_invalida","subcodigo":erro.subcodigo(),"retentavel":false})
                );
            } else {
                eprintln!("IES: listagem offline não concluída [{}]", erro.subcodigo());
            }
            ExitCode::from(1)
        }
    }
}
#[cfg(test)]
mod testes_listagem {
    use super::*;
    #[test]
    fn lista_titulos_distintos_em_ordem_de_captura() {
        let htmls = vec![
            "<title>SEI/UnB - 0007 - Ofício</title>".to_string(),
            "<title>SEI/UnB - 0008 - Memorando</title>".to_string(),
        ];
        let resultado = listar_identidades(&htmls).unwrap();
        assert_eq!(resultado.len(), 2);
        assert_eq!(resultado[0].numero, "0007");
        assert_eq!(resultado[1].tipo, "Memorando");
    }
    #[test]
    fn recusa_identidade_repetida() {
        let html = "<title>SEI/UnB - 0007 - Ofício</title>".to_owned();
        assert_eq!(
            listar_identidades(&[html.clone(), html]),
            Err(ErroReplayDocumento::NumeroRepetido)
        );
    }
    #[test]
    fn recusa_titulo_sem_identidade_do_sei() {
        assert_eq!(
            listar_identidades(&["<title>Portal - 7 - Texto</title>".to_owned()]),
            Err(ErroReplayDocumento::Titulo(
                ErroTituloDocumento::IdentificacaoDaInstalacaoAusenteNoTitulo
            ))
        );
    }
}

fn erro_exportacao(subcodigo: &str, json_mode: bool) -> ExitCode {
    if json_mode {
        println!(
            "{}",
            json!({"codigo":"arquivo_privado_invalido",
            "subcodigo":subcodigo,"retentavel":false})
        );
    } else {
        eprintln!("IES: exportação de texto não concluída [{subcodigo}]");
    }
    ExitCode::from(1)
}
