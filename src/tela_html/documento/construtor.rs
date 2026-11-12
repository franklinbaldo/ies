//! O `TreeSink` que monta a arena do `Documento` durante a leitura.

use super::{Conteudo, DadosDoElemento, Documento, RAIZ, Vertice};
use html5ever::tendril::StrTendril;
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{Attribute, LocalName, QualName, ns};
use std::borrow::Cow;
use std::cell::{Ref, RefCell};
use std::sync::LazyLock;

/// O `TreeSink` que monta a arena durante a leitura.
pub(super) struct Construtor {
    nos: RefCell<Vec<Vertice>>,
    /// Trecho lido em contexto: a raiz é o elemento que o html5ever cria
    /// para contê-lo, o primeiro filho do documento.
    fragmento: bool,
}

/// Nome devolvido ao html5ever para um nó que não é elemento: ele nunca
/// pergunta, mas o tipo exige uma resposta.
static SEM_NOME: LazyLock<QualName> =
    LazyLock::new(|| QualName::new(None, ns!(), LocalName::from("")));

impl Default for Construtor {
    fn default() -> Self {
        Self {
            nos: RefCell::new(vec![Vertice::novo(Conteudo::Raiz)]),
            fragmento: false,
        }
    }
}

impl Construtor {
    pub(super) fn de_fragmento() -> Self {
        Self {
            fragmento: true,
            ..Self::default()
        }
    }
}

impl Vertice {
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
        nos.push(Vertice::novo(conteudo));
        nos.len() - 1
    }

    /// Tira `filho` da lista do pai atual, se houver.
    fn desligar(nos: &mut [Vertice], filho: usize) {
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
                nos.push(Vertice::novo(Conteudo::Texto(texto)));
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
        let nos = self.nos.into_inner();
        let raiz = if self.fragmento {
            nos.get(RAIZ)
                .and_then(|vertice| vertice.filhos.first().copied())
                .unwrap_or(RAIZ)
        } else {
            RAIZ
        };
        Documento { nos, raiz }
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
