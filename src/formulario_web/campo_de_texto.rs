//! Campo de texto livre que o formulário servido publica e que o navegador
//! sempre submete: um `textarea` ou `input` de texto com `name` vai no corpo
//! mesmo vazio. O SEI publica esses campos vazios na tela de cadastro (por
//! exemplo `txaTexto` do marcador e `txaObservacao` do acompanhamento, nas
//! cassettes `marcador-escritas-*` e `processo-acompanhar-*`).
//!
//! As cassettes não gravam o corpo dos POSTs, então não há prova de que o
//! SEI aceite o campo omitido. Sem valor pedido, o campo vai presente e vazio,
//! como o navegador o envia; omitir seria outra escrita, sem evidência.

/// O valor de um campo de texto livre no corpo do POST.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CampoDeTexto<'a> {
    /// O texto pedido, enviado como está.
    Preenchido(&'a str),
    /// Nenhum texto pedido: o campo vai presente e vazio (`nome=`).
    CampoVazio,
}

impl<'a> CampoDeTexto<'a> {
    /// O campo para um valor opcional: sem valor, `CampoVazio`.
    pub(crate) fn de(valor: Option<&'a str>) -> Self {
        valor.map_or(Self::CampoVazio, Self::Preenchido)
    }

    /// O valor que vai no corpo do POST.
    pub(crate) fn enviado(self) -> &'a str {
        match self {
            Self::Preenchido(texto) => texto,
            Self::CampoVazio => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CampoDeTexto;

    #[test]
    fn sem_texto_pedido_o_campo_vai_presente_e_vazio() {
        assert_eq!(CampoDeTexto::de(None), CampoDeTexto::CampoVazio);
        assert_eq!(CampoDeTexto::de(None).enviado(), "");
        assert_eq!(CampoDeTexto::de(Some("Conferir")).enviado(), "Conferir");
    }
}
