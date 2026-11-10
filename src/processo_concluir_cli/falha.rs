//! Contrato compacto das falhas que publicam a situação do processo.
use crate::transporte::StatusHttpRecusado;

/// Campos em ordem histórica; `Some(None)` conserva verificação nula.
#[derive(Default, serde::Serialize)]
pub(crate) struct FalhaDaSituacao<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) aberto_em: Option<&'a [String]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) anotacao_depois: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) candidatos: Option<&'a [String]>,
    codigo: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) depois: Option<&'a [String]>,
    erro: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) falha_do_envio: Option<crate::saida_erro::FalhaPublica>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mensagem_do_servidor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) motivo: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) oferecidos: Option<&'a [String]>,
    retentavel: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_http: Option<u16>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) sugestoes_guardadas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[expect(
        clippy::option_option,
        reason = "o contrato distingue campo ausente de verificação nula"
    )]
    pub(crate) verificacao: Option<Option<&'static str>>,
}

impl<'a> FalhaDaSituacao<'a> {
    pub(crate) fn nova(codigo: &'a str, erro: &'a str, http: Option<StatusHttpRecusado>) -> Self {
        Self {
            codigo,
            erro,
            status_http: http
                .as_ref()
                .map(|h| h.status)
                .filter(|status| *status != 200),
            mensagem_do_servidor: http.and_then(|h| h.mensagem_do_servidor),
            ..Self::default()
        }
    }
}
