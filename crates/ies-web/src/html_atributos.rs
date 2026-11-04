//! Atributos da tag de abertura lidos da fonte original da tag.
//!
//! O `tl` 0.7.8 perde o primeiro caractere do atributo que segue um atributo
//! booleano sem valor (`<input disabled name='n'>` vira `ame`) e indexa `id` e
//! `class` com a mesma chave corrompida. O `tl` continua responsável pela
//! árvore (aninhamento e seleção por elemento); todo atributo lido fora dos
//! testes passa por este módulo, que tokeniza `HTMLTag::raw()` seguindo os
//! estados WHATWG de nome e valor de atributo. Valores saem brutos: decodificar
//! entidades continua sendo decisão de quem lê.
mod busca;
mod tokenizador;

pub use busca::{
    nos_com_id, nos_com_id_em, nos_por_elemento_e_classe,
    nos_por_elemento_e_classe_com_repeticao_identica,
    nos_por_elemento_e_classe_em_todas_as_ocorrencias,
};
pub use tokenizador::tokenizar_tag_de_abertura;

/// Falha ao interpretar a tag de abertura servida; nunca é tratada como
/// atributo ausente pelo chamador.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErroAtributoHtml {
    /// A fonte da tag não começa por `<` seguido do nome do elemento.
    InicioDaTagDeAberturaAusente,
    /// A fonte terminou antes do `>` que fecha a tag de abertura.
    FimDaTagDeAberturaAusente,
    /// Um valor entre aspas não foi fechado antes do fim da fonte.
    ValorEntreAspasSemFechamento,
    /// O atributo pedido aparece mais de uma vez na mesma tag.
    AtributoRepetidoNaTag { atributo: &'static str },
}

impl ErroAtributoHtml {
    #[must_use]
    pub fn subcodigo(self) -> &'static str {
        match self {
            Self::InicioDaTagDeAberturaAusente => "inicio_da_tag_de_abertura_ausente",
            Self::FimDaTagDeAberturaAusente => "fim_da_tag_de_abertura_ausente",
            Self::ValorEntreAspasSemFechamento => "valor_entre_aspas_sem_fechamento",
            Self::AtributoRepetidoNaTag { .. } => "atributo_repetido_na_tag",
        }
    }
}

/// Atributos na ordem da fonte, com nomes em minúsculas ASCII e valores
/// brutos. `None` é atributo sem `=`; `Some("")` é `nome=""`.
#[derive(Debug, PartialEq, Eq)]
pub struct AtributosDaTag<'a> {
    pares: Vec<(String, Option<&'a str>)>,
}

impl<'a> AtributosDaTag<'a> {
    pub(super) fn novo(pares: Vec<(String, Option<&'a str>)>) -> Self {
        Self { pares }
    }

    /// Nomes na ordem da fonte, inclusive repetidos.
    #[cfg(test)]
    #[must_use]
    pub fn nomes(&self) -> impl Iterator<Item = &str> {
        self.pares.iter().map(|(nome, _)| nome.as_str())
    }

    /// Quantos atributos a tag tem, inclusive repetidos.
    #[must_use]
    pub fn total(&self) -> usize {
        self.pares.len()
    }

    /// Quantas vezes `nome` (minúsculo) aparece na tag.
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn ocorrencias(&self, nome: &str) -> usize {
        self.pares.iter().filter(|(chave, _)| chave == nome).count()
    }

    /// Valor bruto do atributo; atributo sem `=` devolve `Ok(None)`, como
    /// ausente. Repetição não escolhe o primeiro nem o último: é recusada.
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn valor(&self, nome: &'static str) -> Result<Option<&'a str>, ErroAtributoHtml> {
        let mut achados = self.pares.iter().filter(|(chave, _)| chave == nome);
        let primeiro = achados.next();
        if achados.next().is_some() {
            return Err(ErroAtributoHtml::AtributoRepetidoNaTag { atributo: nome });
        }
        Ok(primeiro.and_then(|(_, valor)| *valor))
    }

    /// Todos os valores de `nome`, inclusive repetidos, para regras que
    /// recusam quando **qualquer** ocorrência casa (detecção de login).
    #[must_use]
    pub fn valores(&self, nome: &str) -> impl Iterator<Item = &'a str> {
        self.pares
            .iter()
            .filter(move |(chave, _)| chave == nome)
            .filter_map(|(_, valor)| *valor)
    }

    /// Presença de atributo booleano, qualquer que seja a forma do valor
    /// (`disabled`, `disabled=""`, `disabled='disabled'`).
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn presente(&self, nome: &str) -> bool {
        self.ocorrencias(nome) > 0
    }

    /// `class` contém `classe` entre os separadores de espaço HTML.
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn tem_classe(&self, classe: &str) -> Result<bool, ErroAtributoHtml> {
        Ok(self
            .valor("class")?
            .is_some_and(|v| v.split(eh_espaco_html).any(|c| c == classe)))
    }

    /// Como `valor`, mas aceita o atributo repetido quando **todas** as
    /// ocorrências trazem o mesmo valor bruto: a `UnB` 4.0.9 serve
    /// `class="protocoloNormal" … class="protocoloNormal"` no link da
    /// pesquisa. Valores diferentes continuam recusados.
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn valor_com_repeticao_identica(
        &self,
        nome: &'static str,
    ) -> Result<Option<&'a str>, ErroAtributoHtml> {
        let mut achados = self.pares.iter().filter(|(chave, _)| chave == nome);
        let Some((_, primeiro)) = achados.next() else {
            return Ok(None);
        };
        if achados.any(|(_, valor)| valor != primeiro) {
            return Err(ErroAtributoHtml::AtributoRepetidoNaTag { atributo: nome });
        }
        Ok(*primeiro)
    }

    /// `class` (talvez repetido) contém `classe`: falso se nenhuma ocorrência
    /// a contém; erro se umas contêm e outras não.
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn tem_classe_em_todas_as_ocorrencias(
        &self,
        classe: &str,
    ) -> Result<bool, ErroAtributoHtml> {
        let contem: Vec<bool> = self
            .pares
            .iter()
            .filter(|(chave, _)| *chave == "class")
            .map(|(_, valor)| valor.is_some_and(|v| v.split(eh_espaco_html).any(|c| c == classe)))
            .collect();
        match (contem.iter().any(|c| *c), contem.iter().all(|c| *c)) {
            (false, _) => Ok(false),
            (true, true) => Ok(true),
            (true, false) => Err(ErroAtributoHtml::AtributoRepetidoNaTag { atributo: "class" }),
        }
    }

    /// `tem_classe` sobre `valor_com_repeticao_identica("class")`.
    /// # Errors
    /// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
    #[must_use]
    pub fn tem_classe_com_repeticao_identica(
        &self,
        classe: &str,
    ) -> Result<bool, ErroAtributoHtml> {
        Ok(self
            .valor_com_repeticao_identica("class")?
            .is_some_and(|v| v.split(eh_espaco_html).any(|c| c == classe)))
    }
}

pub(super) fn eh_espaco_html(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\u{c}' | '\r' | ' ')
}

/// Tokeniza a tag de abertura de `tag` a partir da fonte original.
pub fn atributos<'t>(
    tag: &'t tl::HTMLTag<'_>,
) -> Result<AtributosDaTag<'t>, ErroAtributoHtml> {
    let fonte = std::str::from_utf8(tag.raw().as_bytes())
        .map_err(|_| ErroAtributoHtml::InicioDaTagDeAberturaAusente)?;
    tokenizar_tag_de_abertura(fonte)
}

/// Valor bruto e próprio de `nome` em `tag`.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
#[must_use]
pub fn valor_do_atributo(
    tag: &tl::HTMLTag<'_>,
    nome: &'static str,
) -> Result<Option<String>, ErroAtributoHtml> {
    Ok(atributos(tag)?.valor(nome)?.map(str::to_owned))
}

/// Presença de atributo booleano em `tag`.
/// # Errors
/// Retorna erro tipado quando a marcação HTML não pode ser interpretada com segurança.
#[must_use]
pub fn atributo_presente(
    tag: &tl::HTMLTag<'_>,
    nome: &str,
) -> Result<bool, ErroAtributoHtml> {
    Ok(atributos(tag)?.presente(nome))
}

#[cfg(test)]
mod tests;
