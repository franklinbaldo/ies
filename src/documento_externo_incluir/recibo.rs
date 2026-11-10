//! #332 — interpretação do recibo do upload temporário, sem HTTP.
//! Oráculo: Todos #775 (`UnB` SEI 4), cassette de escrita `UnB`; a etapa não cadastra documento.
//! Os testes exercitam respostas sintéticas; isto ainda não habilita `--confirmar`.

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReciboUpload {
    pub(crate) nome_upload: String,
    pub(crate) nome: String,
    pub(crate) tipo: String,
    pub(crate) tamanho_bytes: u64,
    pub(crate) data_hora: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum FalhaUpload {
    PreparacaoMultipart(super::multipart::FalhaMultipart),
    Recusado {
        motivo: String,
    },
    /// Depois de um POST, ausência de prova não significa ausência de temporário.
    NaoConfirmado {
        status_http: Option<u16>,
    },
}

pub(crate) fn interpretar_resposta(
    status_http: u16,
    corpo: &str,
    nome_esperado: &str,
    tamanho_esperado: u64,
) -> Result<ReciboUpload, FalhaUpload> {
    // Um POST de upload pode ter criado um temporário mesmo quando o resultado
    // não é reconhecido. Nunca transformar ausência de prova em novo envio.
    let incerto = || FalhaUpload::NaoConfirmado {
        status_http: Some(status_http),
    };
    if status_http != 200 {
        return Err(incerto());
    }

    let texto = corpo.trim();
    if let Some(motivo) = texto.strip_prefix("ERRO#") {
        let motivo = motivo.trim();
        if motivo.is_empty() || motivo.contains('#') {
            return Err(incerto());
        }
        return Err(FalhaUpload::Recusado {
            motivo: motivo.to_owned(),
        });
    }

    // UnB SEI 4: nome_upload#nome#tipo#tamanho#data_hora#
    // Não inventar campos, id do documento nem esquema alternativo.
    let partes: Vec<&str> = texto.split('#').collect();
    let [nome_upload, nome, tipo, tamanho, data_hora, fim] = partes.as_slice() else {
        return Err(incerto());
    };
    let tamanho_bytes = tamanho.parse::<u64>().map_err(|_| incerto())?;
    if !fim.is_empty()
        || nome_upload.is_empty()
        || nome.is_empty()
        || !tipo.eq_ignore_ascii_case("application/pdf")
        || data_hora.is_empty()
        || tamanho_bytes == 0
        || tamanho_bytes != tamanho_esperado
        || *nome != nome_esperado
    {
        return Err(incerto());
    }
    Ok(ReciboUpload {
        nome_upload: (*nome_upload).to_owned(),
        nome: (*nome).to_owned(),
        tipo: (*tipo).to_owned(),
        tamanho_bytes,
        data_hora: (*data_hora).to_owned(),
    })
}

/// Interpreta o retorno do POST temporário, sem confundi-lo com o cadastro.
#[allow(dead_code)] // #332: usado na bancada; a CLI ainda não transmite.
pub(crate) fn interpretar_http<R: std::io::Read>(
    resposta: crate::transporte::RespostaWeb<R>,
    nome_esperado: &str,
    tamanho_esperado: u64,
) -> Result<ReciboUpload, FalhaUpload> {
    let status_http = u16::try_from(resposta.status_code).unwrap_or(0);
    let incerto = || FalhaUpload::NaoConfirmado {
        status_http: Some(status_http),
    };
    if status_http != 200
        || resposta
            .content_length
            .is_some_and(|tamanho| tamanho > 64 * 1024)
    {
        return Err(incerto());
    }
    let bytes =
        crate::transporte::ler_limitado(resposta.corpo, 64 * 1024).map_err(|_| incerto())?;
    let corpo = crate::transporte::corpo::decode_http_body(&bytes, &resposta.mime)
        .map_err(|_| incerto())?;
    interpretar_resposta(status_http, &corpo, nome_esperado, tamanho_esperado)
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECIBO: &str = "94e336aa#prova.pdf#application/pdf#68#11/10/2026 11:07:50#";

    #[test]
    fn aceite_publica_apenas_o_recibo_temporario_observado() {
        // Todos tests/test_documento_juntar_010.py @ caso RECIBO, UnB SEI 4.
        assert_eq!(
            interpretar_resposta(200, RECIBO, "prova.pdf", 68),
            Ok(ReciboUpload {
                nome_upload: "94e336aa".into(),
                nome: "prova.pdf".into(),
                tipo: "application/pdf".into(),
                tamanho_bytes: 68,
                data_hora: "11/10/2026 11:07:50".into(),
            })
        );
    }

    #[test]
    fn erro_expresso_do_servidor_eh_recusa_sem_cadastro() {
        // Todos documento_externo.py: o servidor responde ERRO#motivo.
        assert_eq!(
            interpretar_resposta(200, "ERRO#Formato não aceito", "prova.pdf", 68),
            Err(FalhaUpload::Recusado {
                motivo: "Formato não aceito".into(),
            })
        );
    }

    #[test]
    fn status_inesperado_nao_autoriza_cadastro_ou_retry() {
        for status in [302, 403, 500, 503] {
            assert_eq!(
                interpretar_resposta(status, RECIBO, "prova.pdf", 68),
                Err(FalhaUpload::NaoConfirmado {
                    status_http: Some(status),
                })
            );
        }
    }

    #[test]
    fn corpo_vazio_html_ou_recibo_truncado_eh_nao_confirmado() {
        for corpo in [
            "",
            "<html><title>Login</title></html>",
            "94e336aa#prova.pdf#application/pdf#68#",
            "94e336aa#prova.pdf#application/pdf#68#11/10/2026 11:07:50",
        ] {
            assert_eq!(
                interpretar_resposta(200, corpo, "prova.pdf", 68),
                Err(FalhaUpload::NaoConfirmado {
                    status_http: Some(200),
                })
            );
        }
    }

    #[test]
    fn recibo_temporario_de_outro_mime_nao_autoriza_cadastro() {
        // BDD-332-03: sem recibo comprovando PDF, não cadastrar documento.
        for mime in ["text/html", "image/png", "application/octet-stream"] {
            let diferente = RECIBO.replace("application/pdf", mime);
            assert_eq!(
                interpretar_resposta(200, &diferente, "prova.pdf", 68),
                Err(FalhaUpload::NaoConfirmado {
                    status_http: Some(200)
                }),
                "{mime}"
            );
        }
    }

    #[test]
    fn divergencia_de_nome_ou_tamanho_bloqueia_cadastro() {
        for (nome, tamanho) in [("outro.pdf", 68), ("prova.pdf", 67)] {
            assert_eq!(
                interpretar_resposta(200, RECIBO, nome, tamanho),
                Err(FalhaUpload::NaoConfirmado {
                    status_http: Some(200),
                })
            );
        }
    }

    #[test]
    fn recibo_ambiguo_ou_com_tamanho_nao_numerico_nao_autoriza_cadastro() {
        for corpo in [
            "94e336aa#prova.pdf#application/pdf#xx#11/10/2026 11:07:50#",
            "94e336aa#prova.pdf#application/pdf#68#11/10/2026 11:07:50#extra#",
            "#prova.pdf#application/pdf#68#11/10/2026 11:07:50#",
        ] {
            assert_eq!(
                interpretar_resposta(200, corpo, "prova.pdf", 68),
                Err(FalhaUpload::NaoConfirmado {
                    status_http: Some(200),
                })
            );
        }
    }
}
