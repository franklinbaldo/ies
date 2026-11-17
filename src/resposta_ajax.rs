//! Leitura do XML das respostas AJAX do SEI (`controlador_ajax.php`) com
//! `quick-xml`: uma árvore de elementos com nomes, atributos e texto já
//! decodificados (entidades XML e referências numéricas). Esta fronteira só
//! lê XML bem formado; a forma de cada resposta (quantos elementos, quais
//! atributos, quais valores) é conferida pelo leitor que a consome.
use quick_xml::errors::IllFormedError;
use quick_xml::escape::unescape;
use quick_xml::events::attributes::AttrError;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, XmlVersion};

/// Uma resposta XML inteira, com um único elemento raiz.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RespostaAjax {
    raiz: Elemento,
}

/// Um elemento: nome, atributos com valores decodificados e o texto que
/// vem diretamente dentro dele (sem o de seus filhos).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Elemento {
    nome: String,
    atributos: Vec<(String, String)>,
    texto: String,
    filhos: Vec<Elemento>,
}

/// A resposta que não é XML bem formado de um único elemento raiz.
#[derive(Clone, Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroXml {
    /// Sintaxe, marcação não fechada, atributo malformado ou entidade que o
    /// XML não define; a causa é a do próprio `quick-xml`.
    XmlRecusado(#[source] quick_xml::Error),
    /// Atributo repetido ou sem valor.
    Atributo(#[source] AttrError),
    /// Texto com conteúdo fora do elemento raiz.
    TextoForaDaRaiz,
    /// Fechamento sem abertura correspondente na pilha de elementos.
    FechamentoSemAbertura,
    /// Resposta sem elemento algum.
    SemElementoRaiz,
    /// Mais de um elemento no nível da raiz.
    VariosElementosRaiz,
}

impl RespostaAjax {
    /// Lê `xml` inteiro. Recusa o que o `quick-xml` recusa e o que não tem
    /// exatamente um elemento raiz.
    pub(crate) fn ler(xml: &str) -> Result<Self, ErroXml> {
        let mut leitor = Reader::from_str(xml);
        let mut pilha: Vec<Elemento> = Vec::new();
        let mut raiz: Option<Elemento> = None;
        loop {
            match leitor.read_event().map_err(ErroXml::XmlRecusado)? {
                Event::Start(inicio) => pilha.push(Elemento::abrir(&inicio)?),
                Event::Empty(inicio) => {
                    let elemento = Elemento::abrir(&inicio)?;
                    fechar(elemento, &mut pilha, &mut raiz)?;
                }
                Event::End(_) => {
                    let Some(elemento) = pilha.pop() else {
                        return Err(ErroXml::FechamentoSemAbertura);
                    };
                    fechar(elemento, &mut pilha, &mut raiz)?;
                }
                Event::Text(texto) => acrescentar(&mut pilha, &texto.xml10_content())?,
                Event::CData(dados) => acrescentar(&mut pilha, &dados.xml10_content())?,
                Event::GeneralRef(referencia) => {
                    let trecho = format!("&{};", referencia.xml10_content());
                    let valor = unescape(&trecho)
                        .map_err(|erro| ErroXml::XmlRecusado(quick_xml::Error::Escape(erro)))?;
                    acrescentar(&mut pilha, &valor)?;
                }
                Event::Comment(_) | Event::Decl(_) | Event::PI(_) | Event::DocType(_) => {}
                Event::Eof => break,
            }
        }
        if let Some(aberto) = pilha.last() {
            let causa = IllFormedError::MissingEndTag(aberto.nome.clone());
            return Err(ErroXml::XmlRecusado(causa.into()));
        }
        let raiz = raiz.ok_or(ErroXml::SemElementoRaiz)?;
        Ok(Self { raiz })
    }

    /// Todos os elementos com `nome`, a raiz inclusive, em ordem de documento.
    pub(crate) fn elementos(&self, nome: &str) -> Vec<&Elemento> {
        let mut achados = Vec::new();
        self.raiz.reunir(nome, &mut achados);
        achados
    }
}

impl Elemento {
    fn abrir(inicio: &BytesStart<'_>) -> Result<Self, ErroXml> {
        let nome = inicio.name().as_ref().to_owned();
        let mut atributos = Vec::new();
        for atributo in inicio.attributes() {
            let atributo = atributo.map_err(ErroXml::Atributo)?;
            let chave = atributo.key.as_ref().to_owned();
            let valor = atributo
                .normalized_value(XmlVersion::Implicit1_0)
                .map_err(ErroXml::XmlRecusado)?;
            atributos.push((chave, valor.into_owned()));
        }
        Ok(Self {
            nome,
            atributos,
            texto: String::new(),
            filhos: Vec::new(),
        })
    }

    /// Valor decodificado do atributo `nome`, se o elemento o tiver.
    pub(crate) fn atributo(&self, nome: &str) -> Option<&str> {
        self.atributos
            .iter()
            .find(|(chave, _)| chave == nome)
            .map(|(_, valor)| valor.as_str())
    }

    /// Texto que vem diretamente dentro do elemento, sem aparar espaços.
    pub(crate) fn texto(&self) -> &str {
        &self.texto
    }

    /// Descendentes com `nome` (o próprio elemento não entra), em ordem de documento.
    pub(crate) fn descendentes(&self, nome: &str) -> Vec<&Elemento> {
        let mut achados = Vec::new();
        for filho in &self.filhos {
            filho.reunir(nome, &mut achados);
        }
        achados
    }

    fn reunir<'a>(&'a self, nome: &str, achados: &mut Vec<&'a Elemento>) {
        if self.nome == nome {
            achados.push(self);
        }
        for filho in &self.filhos {
            filho.reunir(nome, achados);
        }
    }
}

/// Um elemento fechado vira filho do que está aberto logo acima ou, se não
/// há nenhum, a raiz (que só pode existir uma vez).
fn fechar(
    elemento: Elemento,
    pilha: &mut [Elemento],
    raiz: &mut Option<Elemento>,
) -> Result<(), ErroXml> {
    if let Some(pai) = pilha.last_mut() {
        pai.filhos.push(elemento);
        return Ok(());
    }
    if raiz.is_some() {
        return Err(ErroXml::VariosElementosRaiz);
    }
    *raiz = Some(elemento);
    Ok(())
}

fn acrescentar(pilha: &mut [Elemento], trecho: &str) -> Result<(), ErroXml> {
    match pilha.last_mut() {
        Some(atual) => {
            atual.texto.push_str(trecho);
            Ok(())
        }
        None if trecho.trim().is_empty() => Ok(()),
        None => Err(ErroXml::TextoForaDaRaiz),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::escape::EscapeError;

    fn lida(xml: &str) -> RespostaAjax {
        RespostaAjax::ler(xml).unwrap()
    }

    #[test]
    fn resolve_entidades_xml_e_referencias_numericas() {
        let resposta = lida(
            "<?xml version='1.0' encoding='iso-8859-1'?>\
             <itens><item id='1' descricao='A &amp; B &lt;C&gt; &#233; &#xE7;'>x &amp; &#65;</item></itens>",
        );
        let itens = resposta.elementos("item");
        assert_eq!(itens.len(), 1);
        assert_eq!(itens[0].atributo("descricao"), Some("A & B <C> é ç"));
        assert_eq!(itens[0].atributo("id"), Some("1"));
        assert_eq!(itens[0].texto(), "x & A");
    }

    #[test]
    fn elementos_incluem_a_raiz_e_descendentes_em_ordem() {
        let resposta = lida("<itens><grupo><item id='1'/></grupo><item id='2'/></itens>");
        let ids: Vec<_> = resposta
            .elementos("item")
            .iter()
            .filter_map(|item| item.atributo("id"))
            .collect();
        assert_eq!(ids, ["1", "2"]);
        assert_eq!(resposta.elementos("itens").len(), 1);
        assert_eq!(resposta.elementos("itens")[0].descendentes("item").len(), 2);
    }

    #[test]
    fn texto_de_cdata_nao_e_decodificado_de_novo() {
        let resposta = lida("<a><![CDATA[1 &amp; 2]]></a>");
        assert_eq!(resposta.elementos("a")[0].texto(), "1 &amp; 2");
    }

    #[test]
    fn recusa_xml_mal_formado_e_formas_fora_de_uma_raiz() {
        assert!(matches!(
            RespostaAjax::ler("<a><b></a>"),
            Err(ErroXml::XmlRecusado(quick_xml::Error::IllFormed(
                IllFormedError::MismatchedEndTag { expected, found }
            ))) if expected == "b" && found == "a"
        ));
        assert!(matches!(
            RespostaAjax::ler("<a>&nbsp;</a>"),
            Err(ErroXml::XmlRecusado(quick_xml::Error::Escape(
                EscapeError::UnrecognizedEntity(_, nome)
            ))) if nome == "nbsp"
        ));
        assert!(matches!(
            RespostaAjax::ler("<a b='1' b='2'/>"),
            Err(ErroXml::Atributo(AttrError::Duplicated(8, 2)))
        ));
        assert!(matches!(
            RespostaAjax::ler("<a/><b/>"),
            Err(ErroXml::VariosElementosRaiz)
        ));
        assert!(matches!(
            RespostaAjax::ler("   "),
            Err(ErroXml::SemElementoRaiz)
        ));
        assert!(matches!(
            RespostaAjax::ler("texto<a/>"),
            Err(ErroXml::TextoForaDaRaiz)
        ));
    }

    #[test]
    fn recusa_marcacao_nao_fechada_no_fim_do_documento() {
        assert!(matches!(
            RespostaAjax::ler("<a><b/>"),
            Err(ErroXml::XmlRecusado(quick_xml::Error::IllFormed(
                IllFormedError::MissingEndTag(nome)
            ))) if nome == "a"
        ));
    }
}
