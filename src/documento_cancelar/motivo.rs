//! A entrada válida de cancelamento: aparada, não vazia e compatível com Latin-1.

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroMotivoDoCancelamento {
    #[causa(nome = "motivo_com_controle")]
    ComControle,
    #[causa(nome = "motivo_vazio")]
    Vazio,
    #[causa(nome = "motivo_fora_da_codificacao")]
    ForaDaCodificacao,
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
        return Err(ErroMotivoDoCancelamento::ComControle);
    }
    let limpo = motivo.trim();
    if limpo.is_empty() {
        return Err(ErroMotivoDoCancelamento::Vazio);
    }
    if !limpo.chars().all(|c| u32::from(c) <= 255) {
        return Err(ErroMotivoDoCancelamento::ForaDaCodificacao);
    }
    Ok(MotivoDoCancelamento(limpo))
}
