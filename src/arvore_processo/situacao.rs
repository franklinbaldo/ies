//! Onde o processo está aberto: o `.html` do nó `PROCESSO` (Refs #59;
//! contracts/cli/processo-obter-situacao.md). Formas observadas na `UnB`
//! 4.0.9, em GO 5.0.3 e na PEN1 5.0.4: "Processo aberto somente na
//! unidade <a>", "Processo aberto nas unidades:<br /><a>…" (com
//! atribuição por unidade) e "Processo não possui andamentos abertos.";
//! nas sete instalações do acervo, "Processo sobrestado na unidade <a>."
//! (sem `aberto_em`: a frase não publica unidades abertas). O texto vem do leitor de literais; as âncoras, do tokenizador de
//! atributos. Qualquer outra forma é "não lida", nunca falha.
use super::vinculo::texto_unico;
use crate::formulario_web::decodificar_atributo;
use crate::html_atributos::atributos;

/// A pessoa a quem o processo está atribuído na unidade: login (texto do
/// link) e nome (`alt` = `title`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Atribuicao {
    pub(crate) sigla: String,
    pub(crate) nome: String,
}

/// Uma unidade onde o processo está aberto: sigla (texto do link), nome
/// (`alt` = `title`) e a atribuição publicada na mesma linha.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnidadeAberta {
    pub(crate) sigla: String,
    pub(crate) nome: String,
    pub(crate) atribuido_a: Option<Atribuicao>,
}

/// A unidade onde o processo está sobrestado: sigla (texto do link) e
/// nome (`alt` = `title`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UnidadeSobrestada {
    pub(crate) sigla: String,
    pub(crate) nome: String,
}

/// Por que `aberto_em` não foi lido.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MotivoAbertoEm {
    RaizPublicaOVinculoDeAnexacao,
    RaizPublicaOSobrestamento,
    HtmlDaRaizNaoPublicado,
    HtmlDaRaizForaDaFormaObservada,
}

impl MotivoAbertoEm {
    pub(crate) fn como_texto(self) -> &'static str {
        match self {
            Self::RaizPublicaOVinculoDeAnexacao => "raiz_publica_o_vinculo_de_anexacao",
            Self::RaizPublicaOSobrestamento => "raiz_publica_o_sobrestamento",
            Self::HtmlDaRaizNaoPublicado => "html_da_raiz_nao_publicado",
            Self::HtmlDaRaizForaDaFormaObservada => "html_da_raiz_fora_da_forma_observada",
        }
    }
}

/// `Unidades(vec![])`: "Processo não possui andamentos abertos.".
/// `Sobrestado`: "Processo sobrestado na unidade <a>."; `aberto_em` sai
/// `null` com [`MotivoAbertoEm::RaizPublicaOSobrestamento`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum AbertoEm {
    Unidades(Vec<UnidadeAberta>),
    Sobrestado(Vec<UnidadeSobrestada>),
    NaoLido(MotivoAbertoEm),
}

/// O `aberto_em` da árvore; nunca falha (um `.html` fora da forma do
/// literal é recusado depois pelo leitor do vínculo).
pub(super) fn ler_aberto_em(html: &str) -> AbertoEm {
    match super::vinculo::html_da_raiz(html) {
        Ok(None) => AbertoEm::NaoLido(MotivoAbertoEm::HtmlDaRaizNaoPublicado),
        Ok(Some(texto)) => ler_unidades(&texto)
            .map(AbertoEm::Unidades)
            .or_else(|| ler_sobrestamento(&texto).map(AbertoEm::Sobrestado))
            .unwrap_or(AbertoEm::NaoLido(
                MotivoAbertoEm::HtmlDaRaizForaDaFormaObservada,
            )),
        Err(_) => AbertoEm::NaoLido(MotivoAbertoEm::HtmlDaRaizForaDaFormaObservada),
    }
}

const SOMENTE: &str = "Processo aberto somente na unidade ";
const NAS_UNIDADES: &str = "Processo aberto nas unidades:";
const NENHUMA: &str = "Processo não possui andamentos abertos.";
const SOBRESTADO: &str = "Processo sobrestado na unidade ";
const ATRIBUIDO: &str = " (atribuído para ";

/// Uma peça do primeiro nível do texto.
#[derive(Debug, PartialEq, Eq)]
enum Peca {
    Texto(String),
    Quebra,
    Ancora { sigla: String, nome: String },
}

/// As unidades do texto do `.html` da raiz, ou `None` fora das formas
/// observadas.
pub(super) fn ler_unidades(texto: &str) -> Option<Vec<UnidadeAberta>> {
    let pecas = pecas(texto)?;
    let unidades = match pecas.as_slice() {
        [Peca::Texto(t), Peca::Quebra] if t == NENHUMA => Vec::new(),
        [Peca::Texto(t), resto @ ..] if t == SOMENTE => {
            let (lida, resto) = unidade(resto)?;
            let fecho = if lida.atribuido_a.is_some() {
                ")."
            } else {
                "."
            };
            match resto {
                [Peca::Texto(t), Peca::Quebra] if t == fecho => vec![lida],
                _ => return None,
            }
        }
        [Peca::Texto(t), Peca::Quebra, resto @ ..] if t == NAS_UNIDADES => plural(resto)?,
        _ => return None,
    };
    let mut siglas: Vec<&str> = unidades.iter().map(|u| u.sigla.as_str()).collect();
    siglas.sort_unstable();
    siglas.dedup();
    (siglas.len() == unidades.len()).then_some(unidades)
}

/// A unidade de "Processo sobrestado na unidade <a>.<br />", ou `None`
/// fora dessa forma. Uma unidade; no SEI 4.1/5 (UFRJ e GO) com
/// " (atribuído para <a>).", que não muda onde está sobrestado.
pub(super) fn ler_sobrestamento(texto: &str) -> Option<Vec<UnidadeSobrestada>> {
    let pecas = pecas(texto)?;
    let [Peca::Texto(frase), resto @ ..] = pecas.as_slice() else {
        return None;
    };
    if frase != SOBRESTADO {
        return None;
    }
    let (lida, fecho) = unidade(resto)?;
    let ponto = if lida.atribuido_a.is_some() {
        ")."
    } else {
        "."
    };
    match fecho {
        [Peca::Texto(t), Peca::Quebra] if t == ponto => Some(vec![UnidadeSobrestada {
            sigla: lida.sigla,
            nome: lida.nome,
        }]),
        _ => None,
    }
}

/// Uma ou mais unidades, cada uma terminada por `<br />` (com atribuição,
/// por `)<br />`), e o `<br />` final.
fn plural(mut resto: &[Peca]) -> Option<Vec<UnidadeAberta>> {
    let mut unidades = Vec::new();
    while resto != [Peca::Quebra] {
        let (lida, depois) = unidade(resto)?;
        resto = match (depois, lida.atribuido_a.is_some()) {
            ([Peca::Quebra, depois @ ..], false) => depois,
            ([Peca::Texto(t), Peca::Quebra, depois @ ..], true) if t == ")" => depois,
            _ => return None,
        };
        unidades.push(lida);
    }
    (!unidades.is_empty()).then_some(unidades)
}

/// A âncora da unidade e, se seguida de ` (atribuído para <âncora>`, a
/// atribuição; o fecho fica para quem chamou.
fn unidade(pecas: &[Peca]) -> Option<(UnidadeAberta, &[Peca])> {
    let [Peca::Ancora { sigla, nome }, resto @ ..] = pecas else {
        return None;
    };
    let (atribuido_a, resto) = match resto {
        [Peca::Texto(t), Peca::Ancora { sigla, nome }, depois @ ..] if t == ATRIBUIDO => {
            let pessoa = Atribuicao {
                sigla: sigla.clone(),
                nome: nome.clone(),
            };
            (Some(pessoa), depois)
        }
        _ => (None, resto),
    };
    let lida = UnidadeAberta {
        sigla: sigla.clone(),
        nome: nome.clone(),
        atribuido_a,
    };
    Some((lida, resto))
}

/// O primeiro nível do texto como peças; `None` para comentário, outra
/// tag ou âncora fora da forma.
fn pecas(texto: &str) -> Option<Vec<Peca>> {
    let dom = tl::parse(texto, tl::ParserOptions::default()).ok()?;
    let parser = dom.parser();
    let mut pecas = Vec::new();
    for no in dom.children() {
        let peca = match no.get(parser)? {
            tl::Node::Raw(bytes) => Peca::Texto(bytes.as_utf8_str().into_owned()),
            tl::Node::Tag(tag) if tag.name() == "br" => {
                let vazia = tag.children().top().len() == 0;
                (vazia && atributos(tag).ok()?.total() == 0).then_some(Peca::Quebra)?
            }
            tl::Node::Tag(tag) if tag.name() == "a" => ancora(tag, parser)?,
            tl::Node::Tag(_) | tl::Node::Comment(_) => return None,
        };
        pecas.push(peca);
    }
    Some(pecas)
}

/// `<a alt="N" title="N" class="ancoraSigla">S</a>`, exatamente.
fn ancora(tag: &tl::HTMLTag<'_>, parser: &tl::Parser<'_>) -> Option<Peca> {
    let lidos = atributos(tag).ok()?;
    if lidos.total() != 3 || lidos.valor("class").ok()? != Some("ancoraSigla") {
        return None;
    }
    let (alt, title) = (lidos.valor("alt").ok()??, lidos.valor("title").ok()??);
    let nome = decodificar_atributo(alt)?;
    if alt != title || nome.is_empty() || nome.trim() != nome {
        return None;
    }
    let sigla = texto_unico(tag, parser);
    if sigla.is_empty() || sigla.trim() != sigla || sigla.contains(['<', '>', '&']) {
        return None;
    }
    Some(Peca::Ancora { sigla, nome })
}
