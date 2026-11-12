//! Atributos da tag de abertura lidos da fonte original da tag.
//!
//! O `tl` 0.7.8 perde o primeiro caractere do atributo que segue um atributo
//! booleano sem valor (`<input disabled name='n'>` vira `ame`) e indexa `id` e
//! `class` com a mesma chave corrompida. O `tl` continua responsável pela
//! árvore (aninhamento e seleção por elemento); todo atributo lido fora dos
//! testes passa por este módulo, que tokeniza `HTMLTag::raw()` seguindo os
//! estados WHATWG de nome e valor de atributo. `valor_do_atributo` devolve o
//! valor bruto; `valor_decodificado` resolve as entidades HTML e recusa as
//! desconhecidas (#751).
mod busca;
mod tokenizador;

pub(crate) use busca::{nos_com_id, nos_com_id_em, nos_por_elemento_e_classe};
pub(crate) use tokenizador::tokenizar_tag_de_abertura;

use crate::formulario_web::decodificar_atributo;

/// Falha ao interpretar a tag de abertura servida; nunca é tratada como
/// atributo ausente pelo chamador.
#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroAtributoHtml {
    /// A fonte da tag não começa por `<` seguido do nome do elemento.
    InicioDaTagDeAberturaAusente,
    /// A fonte terminou antes do `>` que fecha a tag de abertura.
    FimDaTagDeAberturaAusente,
    /// Um valor entre aspas não foi fechado antes do fim da fonte.
    ValorEntreAspasSemFechamento,
    /// Os bytes da fonte da tag não são UTF-8 válido.
    FonteDaTagNaoEhUtf8(#[source] std::str::Utf8Error),
}

/// Atributos na ordem da fonte, com nomes em minúsculas ASCII e valores
/// brutos. `None` é atributo sem `=`; `Some("")` é `nome=""`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct AtributosDaTag<'a> {
    pares: Vec<(String, Option<&'a str>)>,
}

impl<'a> AtributosDaTag<'a> {
    pub(super) fn novo(pares: Vec<(String, Option<&'a str>)>) -> Self {
        Self { pares }
    }

    /// Nomes na ordem da fonte, inclusive repetidos.
    #[cfg(test)]
    pub(crate) fn nomes(&self) -> impl Iterator<Item = &str> {
        self.pares.iter().map(|(nome, _)| nome.as_str())
    }

    /// Quantas vezes `nome` (minúsculo) aparece na tag.
    pub(crate) fn ocorrencias(&self, nome: &str) -> usize {
        self.pares.iter().filter(|(chave, _)| chave == nome).count()
    }

    /// Valor bruto do atributo; atributo sem `=` devolve `None`, como
    /// ausente. Repetido na tag, vale o primeiro, como no navegador: o SEI
    /// repete `class` e `style` com o mesmo valor (#676).
    pub(crate) fn valor(&self, nome: &str) -> Option<&'a str> {
        self.pares
            .iter()
            .find(|(chave, _)| chave == nome)
            .and_then(|(_, valor)| *valor)
    }

    /// Todos os valores de `nome`, inclusive repetidos, para regras que
    /// recusam quando **qualquer** ocorrência casa (detecção de login).
    pub(crate) fn valores(&self, nome: &str) -> impl Iterator<Item = &'a str> {
        self.pares
            .iter()
            .filter(move |(chave, _)| chave == nome)
            .filter_map(|(_, valor)| *valor)
    }

    /// Presença de atributo booleano, qualquer que seja a forma do valor
    /// (`disabled`, `disabled=""`, `disabled='disabled'`).
    pub(crate) fn presente(&self, nome: &str) -> bool {
        self.ocorrencias(nome) > 0
    }

    /// `class` contém `classe` entre os separadores de espaço HTML.
    pub(crate) fn tem_classe(&self, classe: &str) -> bool {
        self.valor("class")
            .is_some_and(|v| v.split(eh_espaco_html).any(|c| c == classe))
    }
}

pub(super) fn eh_espaco_html(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')
}

/// Tokeniza a tag de abertura de `tag` a partir da fonte original.
pub(crate) fn atributos<'t>(
    tag: &'t tl::HTMLTag<'_>,
) -> Result<AtributosDaTag<'t>, ErroAtributoHtml> {
    let fonte =
        std::str::from_utf8(tag.raw().as_bytes()).map_err(ErroAtributoHtml::FonteDaTagNaoEhUtf8)?;
    tokenizar_tag_de_abertura(fonte)
}

/// Valor bruto e próprio de `nome` em `tag`.
pub(crate) fn valor_do_atributo(
    tag: &tl::HTMLTag<'_>,
    nome: &str,
) -> Result<Option<String>, ErroAtributoHtml> {
    Ok(atributos(tag)?.valor(nome).map(str::to_owned))
}

/// Valor de `nome` em `tag` com as entidades HTML resolvidas.
pub(crate) fn valor_decodificado(
    tag: &tl::HTMLTag<'_>,
    nome: &str,
) -> Result<Option<String>, ErroAtributoHtml> {
    Ok(atributos(tag)?.valor(nome).map(decodificar_atributo))
}

/// Presença de atributo booleano em `tag`.
pub(crate) fn atributo_presente(
    tag: &tl::HTMLTag<'_>,
    nome: &str,
) -> Result<bool, ErroAtributoHtml> {
    Ok(atributos(tag)?.presente(nome))
}

/// Insere o booleano sem valor `itemscope` logo após o nome de cada tag de
/// abertura com atributos. No `tl` 0.7.8 isso corrompe o atributo seguinte;
/// cada leitor tem de devolver o mesmo resultado do HTML original.
#[cfg(test)]
pub(crate) fn com_booleano_antes_dos_atributos(html: &str) -> String {
    let bytes = html.as_bytes();
    let mut saida = String::with_capacity(html.len() * 2);
    let mut inicio = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'<' && bytes[i + 1].is_ascii_alphabetic() {
            let mut fim = i + 1;
            while fim < bytes.len() && bytes[fim].is_ascii_alphanumeric() {
                fim += 1;
            }
            if fim < bytes.len() && eh_espaco_html(char::from(bytes[fim])) {
                saida.push_str(&html[inicio..fim]);
                saida.push_str(" itemscope");
                inicio = fim;
            }
            i = fim;
        } else {
            i += 1;
        }
    }
    saida.push_str(&html[inicio..]);
    saida
}

#[cfg(test)]
mod tests;
