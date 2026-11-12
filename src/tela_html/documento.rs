//! A árvore do HTML servido como o navegador a monta (#676): o html5ever
//! decide a estrutura (`<br/>` vazio, `tbody` implícito, texto cru em
//! `textarea`/`script`/`noscript`) e entrega atributos já decodificados,
//! o primeiro valendo em repetição. A árvore é nossa, numa arena; nenhum
//! tipo do html5ever sai deste módulo.

mod construtor;

use construtor::Construtor;
use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::{Attribute, QualName, ns};

/// Índice da raiz (o documento) na arena.
const RAIZ: usize = 0;

/// A árvore inteira de uma tela, ou de um trecho lido como o `innerHTML`
/// de um `body`.
pub(crate) struct Documento {
    nos: Vec<Vertice>,
    /// O documento, ou o elemento que contém o trecho.
    raiz: usize,
}

/// Um nó filho: elemento, texto (já decodificado) ou comentário.
#[derive(Clone, Copy)]
pub(crate) enum No<'d> {
    Elemento(Elemento<'d>),
    Texto(&'d str),
    Comentario,
}

/// Um elemento da árvore, com acesso ao documento que o contém.
#[derive(Clone, Copy)]
pub(crate) struct Elemento<'d> {
    documento: &'d Documento,
    indice: usize,
    dados: &'d DadosDoElemento,
}

struct Vertice {
    pai: Option<usize>,
    filhos: Vec<usize>,
    conteudo: Conteudo,
}

enum Conteudo {
    /// A raiz, ou o conteúdo de um `template`, fora da árvore como no DOM.
    Raiz,
    Elemento(DadosDoElemento),
    Texto(StrTendril),
    Comentario,
}

struct DadosDoElemento {
    nome: QualName,
    atributos: Vec<Attribute>,
    /// O fragmento com o conteúdo, se o elemento é um `template`.
    conteudo_do_modelo: Option<usize>,
}

/// A árvore do HTML servido.
pub(crate) fn arvore(html: &super::HtmlServido) -> Documento {
    html5ever::parse_document(Construtor::default(), html5ever::ParseOpts::default())
        .one(html.as_str())
}

/// Um trecho de HTML (como o que um script põe em `innerHTML`), lido no
/// contexto de um `body`, como o navegador o leria.
pub(crate) fn fragmento(html: &super::HtmlServido) -> Documento {
    let corpo = QualName::new(None, ns!(html), html5ever::local_name!("body"));
    html5ever::parse_fragment(
        Construtor::de_fragmento(),
        html5ever::ParseOpts::default(),
        corpo,
        Vec::new(),
        true,
    )
    .one(html.as_str())
}

impl<'d> No<'d> {
    /// O texto, se o nó é texto.
    pub(crate) fn como_texto(self) -> Option<&'d str> {
        match self {
            No::Texto(texto) => Some(texto),
            No::Elemento(_) | No::Comentario => None,
        }
    }
}

impl Documento {
    /// Os nós do primeiro nível.
    pub(crate) fn nos(&self) -> impl Iterator<Item = No<'_>> {
        self.nos_de(self.raiz)
    }

    /// Os elementos de nome `nome`, na ordem do documento.
    pub(crate) fn elementos<'d, 'n>(
        &'d self,
        nome: &'n str,
    ) -> impl Iterator<Item = Elemento<'d>> + use<'d, 'n> {
        self.descendentes(self.raiz)
            .filter(move |elemento| elemento.nome() == nome)
    }

    /// Os elementos cujo `id` é `id`, na ordem do documento.
    pub(crate) fn com_id<'d, 'i>(
        &'d self,
        id: &'i str,
    ) -> impl Iterator<Item = Elemento<'d>> + use<'d, 'i> {
        self.descendentes(self.raiz)
            .filter(move |elemento| elemento.atributo("id") == Some(id))
    }

    fn nos_de(&self, indice: usize) -> impl Iterator<Item = No<'_>> {
        self.nos
            .get(indice)
            .into_iter()
            .flat_map(|vertice| vertice.filhos.iter())
            .filter_map(|&filho| self.no(filho))
    }

    fn no(&self, indice: usize) -> Option<No<'_>> {
        match &self.nos.get(indice)?.conteudo {
            Conteudo::Elemento(dados) => Some(No::Elemento(Elemento {
                documento: self,
                indice,
                dados,
            })),
            Conteudo::Texto(texto) => Some(No::Texto(texto)),
            Conteudo::Comentario => Some(No::Comentario),
            Conteudo::Raiz => None,
        }
    }

    fn elemento(&self, indice: usize) -> Option<Elemento<'_>> {
        match &self.nos.get(indice)?.conteudo {
            Conteudo::Elemento(dados) => Some(Elemento {
                documento: self,
                indice,
                dados,
            }),
            Conteudo::Raiz | Conteudo::Texto(_) | Conteudo::Comentario => None,
        }
    }

    /// Os índices dos nós abaixo de `topo`, em pré-ordem, sem `topo`.
    fn abaixo(&self, topo: usize) -> impl Iterator<Item = usize> {
        let mut pilha = self.filhos_invertidos(topo);
        std::iter::from_fn(move || {
            let indice = pilha.pop()?;
            pilha.extend(self.filhos_invertidos(indice));
            Some(indice)
        })
    }

    fn filhos_invertidos(&self, indice: usize) -> Vec<usize> {
        self.nos
            .get(indice)
            .map(|no| no.filhos.iter().rev().copied().collect())
            .unwrap_or_default()
    }

    fn descendentes(&self, topo: usize) -> impl Iterator<Item = Elemento<'_>> {
        self.abaixo(topo).filter_map(|indice| self.elemento(indice))
    }
}

impl<'d> Elemento<'d> {
    /// O nome local, em minúsculas para elementos HTML.
    pub(crate) fn nome(&self) -> &'d str {
        &self.dados.nome.local
    }

    /// O valor decodificado do atributo; o primeiro, se repetido.
    pub(crate) fn atributo(&self, nome: &str) -> Option<&'d str> {
        self.dados
            .atributos
            .iter()
            .find(|atributo| atributo.name.ns == ns!() && &*atributo.name.local == nome)
            .map(|atributo| &*atributo.value)
    }

    /// O atributo existe, com qualquer valor (atributo booleano).
    pub(crate) fn presente(self, nome: &str) -> bool {
        self.atributo(nome).is_some()
    }

    /// `class` contém `classe` entre os separadores de espaço do HTML.
    pub(crate) fn tem_classe(self, classe: &str) -> bool {
        self.atributo("class").is_some_and(|valor| {
            valor
                .split(['\t', '\n', '\u{c}', '\r', ' '])
                .any(|nome| nome == classe)
        })
    }

    /// Os elementos descendentes de nome `nome`, na ordem do documento.
    pub(crate) fn elementos<'n>(
        self,
        nome: &'n str,
    ) -> impl Iterator<Item = Elemento<'d>> + use<'d, 'n> {
        self.documento
            .descendentes(self.indice)
            .filter(move |elemento| elemento.nome() == nome)
    }

    /// Os filhos que são elementos, na ordem do documento.
    pub(crate) fn filhos(self) -> impl Iterator<Item = Elemento<'d>> {
        let documento = self.documento;
        documento
            .nos
            .get(self.indice)
            .into_iter()
            .flat_map(|no| no.filhos.iter())
            .filter_map(move |&indice| documento.elemento(indice))
    }

    /// Os nós filhos (elementos, textos e comentários), na ordem.
    pub(crate) fn nos(self) -> impl Iterator<Item = No<'d>> {
        self.documento.nos_de(self.indice)
    }

    /// Quantos atributos o elemento tem (repetidos contam uma vez).
    pub(crate) fn total_de_atributos(self) -> usize {
        self.dados.atributos.len()
    }

    /// O texto de todos os descendentes, concatenado (`textContent`).
    pub(crate) fn texto(&self) -> String {
        self.documento
            .abaixo(self.indice)
            .filter_map(|indice| match &self.documento.nos.get(indice)?.conteudo {
                Conteudo::Texto(texto) => Some(&**texto),
                Conteudo::Raiz | Conteudo::Elemento(_) | Conteudo::Comentario => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{No, arvore};
    use crate::tela_html::HtmlServido;

    fn documento(html: &str) -> super::Documento {
        arvore(&HtmlServido::de_teste(html))
    }

    #[test]
    fn br_autofechado_nao_tem_filhos_e_tbody_e_implicito() {
        let d = documento("<table><tr><td>a<br/><td>b</table>");
        let br = d.elementos("br").next().unwrap();
        assert!(br.elementos("td").next().is_none());
        let tbody = d.elementos("tbody").next().unwrap();
        assert_eq!(tbody.elementos("td").count(), 2);
    }

    #[test]
    fn textarea_e_script_guardam_texto_e_nao_tags() {
        let d = documento("<textarea><input name='a'></textarea><script>'<p>'</script>");
        assert_eq!(d.elementos("input").count(), 0);
        assert_eq!(d.elementos("p").count(), 0);
        let area = d.elementos("textarea").next().unwrap();
        assert_eq!(area.texto(), "<input name='a'>");
    }

    #[test]
    fn atributo_apos_booleano_repetido_e_entidade() {
        let d = documento("<input disabled name='n' name='m' value='a&amp;b' title='&naoexiste;'>");
        let campo = d.elementos("input").next().unwrap();
        assert_eq!(campo.atributo("name"), Some("n"));
        assert_eq!(campo.atributo("disabled"), Some(""));
        assert_eq!(campo.atributo("value"), Some("a&b"));
        assert_eq!(campo.atributo("title"), Some("&naoexiste;"));
        assert_eq!(campo.atributo("id"), None);
    }

    #[test]
    fn filhos_sao_so_os_elementos_do_primeiro_nivel() {
        let d = documento("<table><tr><th>a</th> <td>b<i>c</i></td><!-- x --></tr></table>");
        let linha = d.elementos("tr").next().unwrap();
        let nomes: Vec<_> = linha.filhos().map(|e| e.nome()).collect();
        assert_eq!(nomes, ["th", "td"]);
    }

    #[test]
    fn nos_filhos_trazem_texto_comentario_e_elemento() {
        let d = documento("<div id='d'> a&amp;b <!-- c --><i>x</i></div>");
        let div = d.com_id("d").next().unwrap();
        let nos: Vec<_> = div.nos().collect();
        assert!(matches!(
            nos.as_slice(),
            [No::Texto(" a&b "), No::Comentario, No::Elemento(_)]
        ));
        assert_eq!(nos[0].como_texto(), Some(" a&b "));
        assert_eq!(nos[2].como_texto(), None);
    }

    #[test]
    fn script_guarda_o_conteudo_cru_como_texto() {
        let d = documento("<script><!--//--><![CDATA[//><!--\nf('<b>');\n//--><!]]></script>");
        let script = d.elementos("script").next().unwrap();
        assert_eq!(
            script.texto(),
            "<!--//--><![CDATA[//><!--\nf('<b>');\n//--><!]]>"
        );
        assert_eq!(d.elementos("b").count(), 0);
    }

    #[test]
    fn atributos_presentes_classes_e_total() {
        let d = documento("<table><tr><td class='a  b\tc' disabled class='z'></td></tr></table>");
        let td = d.elementos("td").next().unwrap();
        assert!(td.tem_classe("b") && td.tem_classe("c") && !td.tem_classe("z"));
        assert!(td.presente("disabled") && !td.presente("hidden"));
        assert_eq!(td.total_de_atributos(), 2);
    }

    #[test]
    fn fragmento_le_o_primeiro_nivel_como_innerhtml() {
        let d = super::fragmento(&HtmlServido::de_teste(" Ana<br/>(<a>X</a>)"));
        let nos: Vec<_> = d.nos().collect();
        assert!(matches!(
            nos.as_slice(),
            [No::Texto(" Ana"), No::Elemento(br), No::Texto("("), No::Elemento(_), No::Texto(")")]
                if br.nome() == "br"
        ));
        assert_eq!(d.elementos("a").count(), 1);
    }

    #[test]
    fn conteudo_de_template_fica_fora_da_arvore() {
        let d = documento("<div id='x'><template><p id='x'>t</p></template></div>");
        assert_eq!(d.com_id("x").count(), 1);
        assert_eq!(d.elementos("p").count(), 0);
        assert_eq!(
            d.com_id("x").next().unwrap().elementos("template").count(),
            1
        );
    }

    #[test]
    fn texto_de_tabela_mal_formada_sai_antes_dela() {
        let d = documento("<div><table><tr><td>a</td></tr>x</table></div>");
        assert_eq!(d.elementos("div").next().unwrap().texto(), "xa");
    }

    #[test]
    fn formatacao_mal_aninhada_e_reparentada_na_ordem() {
        let d = documento("<div><b>1<p>2</b>3</p></div>");
        assert_eq!(d.elementos("div").next().unwrap().texto(), "123");
        assert_eq!(d.elementos("b").count(), 2);
        assert_eq!(d.elementos("p").next().unwrap().texto(), "23");
        assert_eq!(d.elementos("p").next().unwrap().nome(), "p");
    }
}
