//! Seleção por número exato a partir de HTML de documento já verificado, sem rede.
use crate::cassette;
use crate::documento::{NumeroDocumento, NumeroDocumentoInvalido};
use crate::documento_web::{ErroTituloDocumento, IdentidadeDoDocumento, identidade_do_titulo};
use serde_json::json;
use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ErroReplayDocumento {
    FonteNaoVerificada,
    Numero(NumeroDocumentoInvalido),
    Titulo(ErroTituloDocumento),
    NumeroNaoObservado,
    NumeroRepetido,
}

impl ErroReplayDocumento {
    fn subcodigo(self) -> &'static str {
        match self {
            Self::FonteNaoVerificada => "replay_documento_nao_selecionado",
            Self::Numero(erro) => erro.subcodigo(),
            Self::Titulo(erro) => erro.subcodigo(),
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
) -> Result<IdentidadeDoDocumento, ErroReplayDocumento> {
    let mut encontrado = None;
    for html in htmls {
        let documento = identidade_do_titulo(html).map_err(ErroReplayDocumento::Titulo)?;
        if documento.numero == numero.as_str() {
            if encontrado.is_some() {
                return Err(ErroReplayDocumento::NumeroRepetido);
            }
            encontrado = Some(documento);
        }
    }
    encontrado.ok_or(ErroReplayDocumento::NumeroNaoObservado)
}

fn ler(
    card: &Path,
    arquivo: &Path,
    passos: &[usize],
    numero: Option<&str>,
) -> Result<(IdentidadeDoDocumento, String), ErroReplayDocumento> {
    if let Some(numero) = numero {
        let solicitado = NumeroDocumento::novo(numero).map_err(ErroReplayDocumento::Numero)?;
        let (paginas, instalacao) = cassette::read_cassette_documents(card, arquivo, passos)
            .map_err(|_| ErroReplayDocumento::FonteNaoVerificada)?;
        let documento = selecionar(&paginas, &solicitado)?;
        Ok((documento, instalacao))
    } else {
        let (html, instalacao) = cassette::read_cassette_document(card, arquivo, passos)
            .map_err(|_| ErroReplayDocumento::FonteNaoVerificada)?;
        let documento = identidade_do_titulo(&html).map_err(ErroReplayDocumento::Titulo)?;
        Ok((documento, instalacao))
    }
}

pub(super) fn mostrar(
    card: &Path,
    arquivo: &Path,
    passos: &[usize],
    numero: Option<&str>,
    json_mode: bool,
) -> ExitCode {
    match ler(card, arquivo, passos, numero) {
        Ok((documento, instalacao)) => {
            if json_mode {
                println!(
                    "{}",
                    json!({
                        "documento": documento,
                        "instalacao": instalacao,
                        "escopo": "titulo_documento_offline"
                    })
                );
            } else {
                println!(
                    "Documento SEI {}: {} (título de HTML offline)",
                    documento.numero, documento.tipo
                );
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
