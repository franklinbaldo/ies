//! #332 — interpretação do recibo do upload temporário, sem HTTP.
//! Oráculo: Todos #775 (UnB SEI 4), cassette de escrita UnB; a etapa não cadastra documento.
//! Os testes exercitam respostas sintéticas; isto ainda não habilita `--confirmar`.

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) struct ReciboUpload {
    pub(crate) nome_upload: String,
    pub(crate) nome: String,
    pub(crate) tipo: String,
    pub(crate) tamanho_bytes: u64,
    pub(crate) data_hora: String,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub(crate) enum FalhaUpload {
    Recusado { motivo: String },
    /// Depois de um POST, ausência de prova não significa ausência de temporário.
    EfeitoIncerto { status_http: Option<u16> },
}

pub(crate) fn interpretar_resposta(
    status_http: u16,
    _corpo: &str,
    _nome_esperado: &str,
    _tamanho_esperado: u64,
) -> Result<ReciboUpload, FalhaUpload> {
    Err(FalhaUpload::EfeitoIncerto {
        status_http: Some(status_http),
    })
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
                Err(FalhaUpload::EfeitoIncerto {
                    status_http: Some(status),
                })
            );
        }
    }

    #[test]
    fn corpo_vazio_html_ou_recibo_truncado_eh_efeito_incerto() {
        for corpo in [
            "",
            "<html><title>Login</title></html>",
            "94e336aa#prova.pdf#application/pdf#68#",
            "94e336aa#prova.pdf#application/pdf#68#11/10/2026 11:07:50",
        ] {
            assert_eq!(
                interpretar_resposta(200, corpo, "prova.pdf", 68),
                Err(FalhaUpload::EfeitoIncerto {
                    status_http: Some(200),
                })
            );
        }
    }

    #[test]
    fn divergencia_de_nome_ou_tamanho_bloqueia_cadastro() {
        for (nome, tamanho) in [("outro.pdf", 68), ("prova.pdf", 67)] {
            assert_eq!(
                interpretar_resposta(200, RECIBO, nome, tamanho),
                Err(FalhaUpload::EfeitoIncerto {
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
                Err(FalhaUpload::EfeitoIncerto {
                    status_http: Some(200),
                })
            );
        }
    }
}
