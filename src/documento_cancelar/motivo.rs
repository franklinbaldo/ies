//! A entrada válida de cancelamento: aparada, não vazia e compatível com Latin-1.

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "tipo")]
pub(crate) enum ErroMotivoDoCancelamento {
    CaractereDeControleNoMotivo,
    MotivoVazio,
    MotivoNaoRepresentavelEmLatin1,
}

impl ErroMotivoDoCancelamento {
    pub(crate) fn mensagem(self) -> &'static str {
        match self {
            Self::CaractereDeControleNoMotivo => "o motivo contém caractere de controle",
            Self::MotivoVazio => "o motivo está vazio",
            Self::MotivoNaoRepresentavelEmLatin1 => "o motivo não é representável em ISO-8859-1",
        }
    }
}

/// Só a validação deste módulo pode construir um motivo para o pedido.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MotivoDoCancelamento<'a>(&'a str);

impl<'a> MotivoDoCancelamento<'a> {
    pub(crate) fn as_str(self) -> &'a str {
        self.0
    }
}

pub(crate) fn conferir_motivo(
    motivo: &str,
) -> Result<MotivoDoCancelamento<'_>, ErroMotivoDoCancelamento> {
    if motivo.chars().any(char::is_control) {
        return Err(ErroMotivoDoCancelamento::CaractereDeControleNoMotivo);
    }
    let limpo = motivo.trim();
    if limpo.is_empty() {
        return Err(ErroMotivoDoCancelamento::MotivoVazio);
    }
    if !limpo.chars().all(|c| u32::from(c) <= 255) {
        return Err(ErroMotivoDoCancelamento::MotivoNaoRepresentavelEmLatin1);
    }
    Ok(MotivoDoCancelamento(limpo))
}
