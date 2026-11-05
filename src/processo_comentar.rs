//! Verificação offline do efeito de um único POST de comentário.
//! A resposta HTTP isolada nunca comprova a escrita; a conferência requer
//! o id do aceite, texto exato e lista relida e completa.
use crate::erro_html::ErroHtml;
use crate::processo_comentarios::ComentariosLidos;

#[derive(Clone, Copy, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub enum FalhaDaConferencia {
    ComentarioNaoVisivelNaReleitura,
    #[causa(nome = "texto_do_comentario_diferente_do_pedido")]
    TextoDiferenteDoPedido,
    #[causa(transparente)]
    Html(ErroHtml),
}

impl FalhaDaConferencia {
    #[must_use]
    pub fn subcodigo(self) -> &'static str {
        crate::erro_publico::folha(&self)
    }
}

/// Só uma lista relida com uma ocorrência de `id_aceite` e texto idêntico prova
/// o efeito. Sem id ou sem leitura, a CLI futura deverá sair com código 3;
/// não é permitido reenviar para descobrir se o POST anterior funcionou.
///
/// # Errors
///
/// Retorna uma condição tipada quando o id aceito não é inequívoco, está
/// ausente da lista relida ou seu texto não corresponde exatamente ao pedido.
pub fn conferir_efeito(
    relidos: &ComentariosLidos,
    id_aceite: &str,
    texto_pedido: &str,
) -> Result<(), FalhaDaConferencia> {
    if id_aceite.is_empty() || !id_aceite.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ErroHtml::ausente("id_do_comentario_ausente_no_aceite").into());
    }
    let mut achados = relidos.itens.iter().filter(|item| item.id == id_aceite);
    let item = achados
        .next()
        .ok_or(FalhaDaConferencia::ComentarioNaoVisivelNaReleitura)?;
    if achados.next().is_some() {
        return Err(ErroHtml::repetido("comentario_novo_repetido_na_releitura").into());
    }
    if item.texto != texto_pedido {
        return Err(FalhaDaConferencia::TextoDiferenteDoPedido);
    }
    Ok(())
}

#[cfg(test)]
#[path = "processo_comentar/tests.rs"]
mod tests;
