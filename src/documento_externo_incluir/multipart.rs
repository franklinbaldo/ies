//! #332 — multipart do PDF conforme o upload `UnB` observado no Todos #775.
//! Etapa offline: o corpo será consumido pelo transporte somente depois da prévia validada.
use crate::formulario_web::CodificacaoDaTela;
use std::io::{Cursor, Read};

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum FalhaMultipart {
    NomeIncompativelComCharset,
    FronteiraInvalida,
    TamanhoInvalido,
    /// O nome do arquivo tem um caractere que o ISO-8859-1 da tela não representa.
    CaractereDoNomeForaDoIso88591(char),
}

/// Peças de um único POST de upload. O PDF fica no leitor, não em memória.
pub(crate) struct MultipartPdf {
    cabecalho: Vec<u8>,
    rodape: Vec<u8>,
    tamanho_total: u64,
    tipo_conteudo: String,
}

impl MultipartPdf {
    pub(crate) fn tamanho_total(&self) -> u64 {
        self.tamanho_total
    }

    pub(crate) fn tipo_conteudo(&self) -> &str {
        &self.tipo_conteudo
    }

    /// Encadeia partes sem copiar o conteúdo inteiro do PDF para memória.
    pub(crate) fn corpo<'a, R: Read + 'a>(&'a self, arquivo: R) -> impl Read + 'a {
        Cursor::new(self.cabecalho.as_slice())
            .chain(arquivo)
            .chain(Cursor::new(self.rodape.as_slice()))
    }
}

/// Usa a codificação da tela servida, não uma codificação escolhida pelo cliente.
/// Recebe apenas um nome já inspecionado e um boundary gerado pela operação.
pub(crate) fn preparar_pdf(
    nome: &str,
    tamanho_bytes: u64,
    fronteira: &str,
    codificacao: CodificacaoDaTela,
) -> Result<MultipartPdf, FalhaMultipart> {
    if fronteira.is_empty()
        || !fronteira
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
    {
        return Err(FalhaMultipart::FronteiraInvalida);
    }
    let mut nome_codificado = Vec::with_capacity(nome.len());
    if nome.is_empty() {
        return Err(FalhaMultipart::NomeIncompativelComCharset);
    }
    for caractere in nome.chars() {
        if caractere == '"' {
            // Mesmo escape de aspas do browser observado no Todos.
            nome_codificado.extend_from_slice(b"%22");
            continue;
        }
        if caractere.is_control() {
            return Err(FalhaMultipart::NomeIncompativelComCharset);
        }
        match codificacao {
            CodificacaoDaTela::Iso88591 => {
                let codigo = u32::from(caractere);
                let byte = u8::try_from(codigo)
                    .map_err(|_| FalhaMultipart::CaractereDoNomeForaDoIso88591(caractere))?;
                nome_codificado.push(byte);
            }
            CodificacaoDaTela::Utf8 => {
                let mut buffer = [0u8; 4];
                nome_codificado.extend_from_slice(caractere.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    let mut cabecalho = format!(
        "--{fronteira}\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\""
    )
    .into_bytes();
    cabecalho.extend_from_slice(&nome_codificado);
    cabecalho.extend_from_slice(b"\"\r\nContent-Type: application/pdf\r\n\r\n");
    let rodape = format!("\r\n--{fronteira}--\r\n").into_bytes();
    let tamanho_total = u64::try_from(cabecalho.len())
        .ok()
        .and_then(|inicio| inicio.checked_add(tamanho_bytes))
        .and_then(|parcial| parcial.checked_add(u64::try_from(rodape.len()).ok()?))
        .ok_or(FalhaMultipart::TamanhoInvalido)?;
    Ok(MultipartPdf {
        cabecalho,
        rodape,
        tamanho_total,
        tipo_conteudo: format!("multipart/form-data; boundary={fronteira}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    const PDF: &[u8] = b"%PDF-1.4\n%%EOF\n";

    #[test]
    fn multipart_pdf_tem_tamanho_exato_e_corpo_sem_chunked() {
        // Todos tests/test_documento_juntar_010.py: filArquivo, Content-Length conhecido.
        let plano = preparar_pdf(
            "prova.pdf",
            PDF.len() as u64,
            "----ies332",
            CodificacaoDaTela::Iso88591,
        )
        .unwrap();
        assert_eq!(
            plano.tipo_conteudo(),
            "multipart/form-data; boundary=----ies332"
        );
        let mut corpo = Vec::new();
        plano.corpo(PDF).read_to_end(&mut corpo).unwrap();
        assert_eq!(plano.tamanho_total(), corpo.len() as u64);
        assert_eq!(
            corpo,
            [
                b"------ies332\r\nContent-Disposition: form-data; name=\"filArquivo\"; filename=\"prova.pdf\"\r\nContent-Type: application/pdf\r\n\r\n".as_slice(),
                PDF,
                b"\r\n------ies332--\r\n".as_slice()
            ]
            .concat()
        );
    }

    #[test]
    fn nome_do_pdf_usa_charset_da_tela_sem_corromper_acentos() {
        // Todos #775: UnB serviu ISO-8859-1; UTF-8 deve seguir tela UTF-8.
        let latin = preparar_pdf(
            "ficção.pdf",
            PDF.len() as u64,
            "----ies332",
            CodificacaoDaTela::Iso88591,
        )
        .unwrap();
        let utf8 = preparar_pdf(
            "ficção.pdf",
            PDF.len() as u64,
            "----ies332",
            CodificacaoDaTela::Utf8,
        )
        .unwrap();
        let mut l = Vec::new();
        let mut u = Vec::new();
        latin.corpo(PDF).read_to_end(&mut l).unwrap();
        utf8.corpo(PDF).read_to_end(&mut u).unwrap();
        assert!(l.windows(10).any(|v| v == b"fic\xe7\xe3o.pdf"));
        assert!(
            u.windows("ficção.pdf".len())
                .any(|v| v == "ficção.pdf".as_bytes())
        );
    }

    #[test]
    fn nome_irrepresentavel_no_charset_recusa_antes_de_enviar() {
        assert!(matches!(
            preparar_pdf(
                "prova-📄.pdf",
                PDF.len() as u64,
                "----ies332",
                CodificacaoDaTela::Iso88591
            ),
            Err(FalhaMultipart::CaractereDoNomeForaDoIso88591('📄'))
        ));
    }

    #[test]
    fn fronteira_malformada_tamanho_impossivel_e_aspas_no_nome() {
        assert!(matches!(
            preparar_pdf("a.pdf", 1, "xx\r\nX: 1", CodificacaoDaTela::Utf8),
            Err(FalhaMultipart::FronteiraInvalida)
        ));
        assert!(matches!(
            preparar_pdf("a.pdf", u64::MAX, "----ies332", CodificacaoDaTela::Utf8),
            Err(FalhaMultipart::TamanhoInvalido)
        ));
        let plano = preparar_pdf(
            "a\"b.pdf",
            PDF.len() as u64,
            "----ies332",
            CodificacaoDaTela::Utf8,
        )
        .unwrap();
        let mut corpo = Vec::new();
        plano.corpo(PDF).read_to_end(&mut corpo).unwrap();
        assert!(
            corpo
                .windows(b"filename=\"a%22b.pdf\"".len())
                .any(|p| p == b"filename=\"a%22b.pdf\"")
        );
    }
}
