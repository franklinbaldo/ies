//! A árvore do HTML servido como o navegador a monta (#676): o html5ever
//! decide a estrutura (`<br/>` vazio, `tbody` implícito, texto cru em
//! `textarea`/`script`/`noscript`) e entrega atributos já decodificados,
//! o primeiro valendo em repetição. A árvore é nossa, numa arena; nenhum
//! tipo do html5ever sai deste módulo.

use html5ever::tendril::{StrTendril, TendrilSink};
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{Attribute, LocalName, QualName, ns};
use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::sync::LazyLock;

/// Índice da raiz (o documento) na arena.
const RAIZ: usize = 0;

/// A árvore inteira de uma tela.
pub(crate) struct Documento {
    nos: Vec<No>,
}

/// Um elemento da árvore, com acesso ao documento que o contém.
#[derive(Clone, Copy)]
pub(crate) struct Elemento<'d> {
    documento: &'d Documento,
    indice: usize,
    dados: &'d DadosDoElemento,
}

struct No {
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

impl Documento {
    /// Os elementos de nome `nome`, na ordem do documento.
    pub(crate) fn elementos<'d>(&'d self, nome: &'d str) -> impl Iterator<Item = Elemento<'d>> {
        self.descendentes(RAIZ)
            .filter(move |elemento| elemento.nome() == nome)
    }

    /// Os elementos cujo `id` é `id`, na ordem do documento.
    pub(crate) fn com_id<'d>(&'d self, id: &'d str) -> impl Iterator<Item = Elemento<'d>> {
        self.descendentes(RAIZ)
            .filter(move |elemento| elemento.atributo("id") == Some(id))
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

    /// Os elementos descendentes de nome `nome`, na ordem do documento.
    pub(crate) fn elementos(self, nome: &'d str) -> impl Iterator<Item = Elemento<'d>> {
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

/// O `TreeSink` que monta a arena durante a leitura.
struct Construtor {
    nos: RefCell<Vec<No>>,
}

/// Nome devolvido ao html5ever para um nó que não é elemento: ele nunca
/// pergunta, mas o tipo exige uma resposta.
static SEM_NOME: LazyLock<QualName> =
    LazyLock::new(|| QualName::new(None, ns!(), LocalName::from("")));

impl Default for Construtor {
    fn default() -> Self {
        Self {
            nos: RefCell::new(vec![No::novo(Conteudo::Raiz)]),
        }
    }
}

impl No {
    fn novo(conteudo: Conteudo) -> Self {
        Self {
            pai: None,
            filhos: Vec::new(),
            conteudo,
        }
    }
}

impl Construtor {
    fn criar(&self, conteudo: Conteudo) -> usize {
        let mut nos = self.nos.borrow_mut();
        nos.push(No::novo(conteudo));
        nos.len() - 1
    }

    /// Tira `filho` da lista do pai atual, se houver.
    fn desligar(nos: &mut [No], filho: usize) {
        let Some(pai) = nos.get(filho).and_then(|no| no.pai) else {
            return;
        };
        if let Some(no) = nos.get_mut(pai) {
            no.filhos.retain(|&f| f != filho);
        }
        if let Some(no) = nos.get_mut(filho) {
            no.pai = None;
        }
    }

    /// Insere `novo` em `pai`, antes de `antes_de` ou no fim; texto logo
    /// após texto é concatenado, como pede o `TreeSink`.
    fn inserir(&self, pai: usize, antes_de: Option<usize>, novo: NodeOrText<usize>) {
        let mut nos = self.nos.borrow_mut();
        if let NodeOrText::AppendNode(filho) = &novo {
            Self::desligar(&mut nos, *filho);
        }
        let Some(filhos) = nos.get(pai).map(|no| &no.filhos) else {
            return;
        };
        let posicao = antes_de
            .and_then(|irmao| filhos.iter().position(|&f| f == irmao))
            .unwrap_or(filhos.len());
        let anterior = posicao.checked_sub(1).and_then(|i| filhos.get(i)).copied();
        let filho = match novo {
            NodeOrText::AppendNode(filho) => filho,
            NodeOrText::AppendText(texto) => {
                if let Some(Conteudo::Texto(existente)) = anterior
                    .and_then(|i| nos.get_mut(i))
                    .map(|no| &mut no.conteudo)
                {
                    existente.push_tendril(&texto);
                    return;
                }
                nos.push(No::novo(Conteudo::Texto(texto)));
                nos.len() - 1
            }
        };
        if let Some(no) = nos.get_mut(pai) {
            no.filhos.insert(posicao, filho);
        }
        if let Some(no) = nos.get_mut(filho) {
            no.pai = Some(pai);
        }
    }

    fn pai(&self, no: usize) -> Option<usize> {
        self.nos.borrow().get(no)?.pai
    }
}

impl TreeSink for Construtor {
    type Handle = usize;
    type Output = Documento;
    type ElemName<'a> = Ref<'a, QualName>;

    fn finish(self) -> Documento {
        Documento {
            nos: self.nos.into_inner(),
        }
    }

    fn parse_error(&self, _mensagem: Cow<'static, str>) {}

    fn get_document(&self) -> usize {
        RAIZ
    }

    fn elem_name<'a>(&'a self, alvo: &'a usize) -> Ref<'a, QualName> {
        Ref::map(self.nos.borrow(), |nos| {
            if let Some(Conteudo::Elemento(dados)) = nos.get(*alvo).map(|no| &no.conteudo) {
                &dados.nome
            } else {
                &SEM_NOME
            }
        })
    }

    fn create_element(
        &self,
        nome: QualName,
        atributos: Vec<Attribute>,
        flags: ElementFlags,
    ) -> usize {
        let conteudo_do_modelo = flags.template.then(|| self.criar(Conteudo::Raiz));
        self.criar(Conteudo::Elemento(DadosDoElemento {
            nome,
            atributos,
            conteudo_do_modelo,
        }))
    }

    fn create_comment(&self, _texto: StrTendril) -> usize {
        self.criar(Conteudo::Comentario)
    }

    fn create_pi(&self, _alvo: StrTendril, _dados: StrTendril) -> usize {
        self.criar(Conteudo::Comentario)
    }

    fn append(&self, pai: &usize, filho: NodeOrText<usize>) {
        self.inserir(*pai, None, filho);
    }

    fn append_based_on_parent_node(
        &self,
        elemento: &usize,
        anterior: &usize,
        filho: NodeOrText<usize>,
    ) {
        match self.pai(*elemento) {
            Some(pai) => self.inserir(pai, Some(*elemento), filho),
            None => self.inserir(*anterior, None, filho),
        }
    }

    fn append_doctype_to_document(
        &self,
        _nome: StrTendril,
        _publico: StrTendril,
        _sistema: StrTendril,
    ) {
    }

    fn get_template_contents(&self, alvo: &usize) -> usize {
        let nos = self.nos.borrow();
        match nos.get(*alvo).map(|no| &no.conteudo) {
            Some(Conteudo::Elemento(dados)) => dados.conteudo_do_modelo.unwrap_or(*alvo),
            Some(Conteudo::Raiz | Conteudo::Texto(_) | Conteudo::Comentario) | None => *alvo,
        }
    }

    fn same_node(&self, x: &usize, y: &usize) -> bool {
        x == y
    }

    fn set_quirks_mode(&self, _modo: QuirksMode) {}

    fn append_before_sibling(&self, irmao: &usize, novo: NodeOrText<usize>) {
        if let Some(pai) = self.pai(*irmao) {
            self.inserir(pai, Some(*irmao), novo);
        }
    }

    fn add_attrs_if_missing(&self, alvo: &usize, atributos: Vec<Attribute>) {
        let mut nos = self.nos.borrow_mut();
        if let Some(Conteudo::Elemento(dados)) = nos.get_mut(*alvo).map(|no| &mut no.conteudo) {
            for atributo in atributos {
                if !dados.atributos.iter().any(|a| a.name == atributo.name) {
                    dados.atributos.push(atributo);
                }
            }
        }
    }

    fn remove_from_parent(&self, alvo: &usize) {
        Self::desligar(&mut self.nos.borrow_mut(), *alvo);
    }

    fn reparent_children(&self, origem: &usize, destino: &usize) {
        let filhos = self
            .nos
            .borrow_mut()
            .get_mut(*origem)
            .map(|no| std::mem::take(&mut no.filhos))
            .unwrap_or_default();
        for filho in filhos {
            self.inserir(*destino, None, NodeOrText::AppendNode(filho));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::arvore;
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
