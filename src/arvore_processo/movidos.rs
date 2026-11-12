//! Documentos movidos para outro processo (Refs #59;
//! contracts/cli/processo-obter-arvore.md). Forma observada na PEN1 5.0.4
//! (04/11/2026): `DOCUMENTO_MOVIDO` com 16 argumentos, a1 composto
//! `<id_documento>-<id_rel_protocolo_protocolo>`, a2 o processo, a3 o
//! `arvore_visualizar` do próprio processo e a15 o número SEI; o `.html` do
//! nó tem um único `<a>`, o `procedimento_trabalhar` assinado do destino
//! (com o `id_documento` da primeira parte de a1), cujo texto é o NUP do
//! destino. A dica (a6) e a barra não são lidas; a frase não classifica.
//! Primeira parte igual ao id de um `DOCUMENTO` (ida e volta, cassettes
//! `documento-mover-cancelar-*`) é `presente_entre_os_documentos`.
use super::endereco::conferir_endereco_relativo;
use super::literais::{Literal, indice_do_html, ler_declaracao, ler_html};
use super::vinculo::{pares, texto_unico};
use super::{
    ARGUMENTOS, ArvoreLida, DestinoDoMovido, DocumentoLido, DocumentoMovido, EnderecoDaArvore,
    ErroArvore, ProcessoNaArvore,
};
use crate::acao_do_sei::AcaoDoSei;
use crate::consulta_do_controlador::ConsultaDoControlador;
use crate::instalacao::Instalacao;
use std::collections::{HashMap, HashSet};

const TIPO: &str = "DOCUMENTO_MOVIDO";

/// Um movido lido, com o pai (a2) ainda não conferido.
pub(super) struct MovidoLido {
    pai: String,
    movido: DocumentoMovido,
}

pub(super) fn ler_movido(a: &[Literal; ARGUMENTOS]) -> Result<MovidoLido, ErroArvore> {
    let fora = || ErroArvore::DocumentoMovidoForaDaFormaObservada;
    let id = super::texto(&a[1])?;
    if documento_do_composto(id).is_none() {
        return Err(fora());
    }
    let pai = super::texto(&a[2])?;
    // a3 acompanha o pai: um a3 de outro processo é pai divergente.
    if !super::decimal(pai) {
        return Err(ErroArvore::DocumentoMovidoComPaiDiferenteDoProcesso);
    }
    if !endereco_observado(super::texto(&a[3])?, pai) {
        return Err(fora());
    }
    let rotulo = super::texto(&a[5])?;
    if rotulo.is_empty() {
        return Err(fora());
    }
    Ok(MovidoLido {
        pai: pai.to_owned(),
        movido: DocumentoMovido {
            id: id.to_owned(),
            rotulo: rotulo.to_owned(),
            numero_sei: super::texto_ou_nulo(&a[15])?.map(str::to_owned),
            destino: None,
            presente_entre_os_documentos: false,
        },
    })
}

/// A primeira parte de `<decimal>-<decimal>`.
fn documento_do_composto(id: &str) -> Option<&str> {
    let (documento, relacao) = id.split_once('-')?;
    (super::decimal(documento) && super::decimal(relacao)).then_some(documento)
}

/// `controlador.php?…` com uma `acao=arvore_visualizar`, um
/// `id_procedimento` igual ao pai e nenhum `id_documento`.
fn endereco_observado(url: &str, pai: &str) -> bool {
    let Some(consulta) = url.strip_prefix("controlador.php?") else {
        return false;
    };
    ConsultaDoControlador::ler(consulta).is_ok_and(|consulta| {
        consulta.acao() == Ok(AcaoDoSei::ArvoreVisualizar)
            && consulta.unico("id_procedimento") == Ok(pai)
            && consulta.valores("id_documento").next().is_none()
    })
}

/// Confere os movidos contra o processo e marca os cuja primeira parte é
/// id de um `DOCUMENTO` da árvore; devolve-os na ordem da árvore.
pub(super) fn conferir_movidos(
    processo: &ProcessoNaArvore,
    lidos: Vec<MovidoLido>,
    documentos: &[DocumentoLido],
) -> Result<Vec<DocumentoMovido>, ErroArvore> {
    if lidos.iter().any(|l| l.pai != processo.id) {
        return Err(ErroArvore::DocumentoMovidoComPaiDiferenteDoProcesso);
    }
    let mut vistos: HashSet<&str> = HashSet::new();
    for lido in &lidos {
        let documento = documento_do_composto(&lido.movido.id).unwrap_or_default();
        if !vistos.insert(documento) {
            return Err(ErroArvore::DocumentoMovidoRepetidoNaArvore);
        }
    }
    Ok(lidos
        .into_iter()
        .map(|l| {
            let documento = documento_do_composto(&l.movido.id).unwrap_or_default();
            DocumentoMovido {
                presente_entre_os_documentos: documentos
                    .iter()
                    .any(|d| d.documento.id == documento),
                ..l.movido
            }
        })
        .collect())
}

/// Preenche o destino de cada movido cujo `.html` publica o link.
pub(crate) fn ler_destinos_dos_movidos(
    instalacao: &Instalacao,
    html: &str,
    arvore: &mut ArvoreLida,
) -> Result<(), ErroArvore> {
    let repetidos = ErroArvore::DestinosDoDocumentoMovidoRepetidos;
    // Índice do nó → id composto, só dos movidos.
    let mut nos: HashMap<usize, String> = HashMap::new();
    let mut lidos: HashMap<String, Option<DestinoDoMovido>> = HashMap::new();
    for linha in html.split('\n') {
        if let Some((i, argumentos)) = ler_declaracao(linha)?
            && argumentos.first() == Some(&Literal::Texto(TIPO.into()))
            && let Some(Literal::Texto(id)) = argumentos.get(1)
        {
            nos.insert(i, id.clone());
        }
        let Some(id) = indice_do_html(linha).and_then(|i| nos.get(&i)) else {
            continue;
        };
        let Some((_, texto)) =
            ler_html(linha).map_err(|e| ErroArvore::LinhaDoDestinoMovidoIlegivel(Box::new(e)))?
        else {
            continue;
        };
        let destino = destino_do_html(instalacao, &texto, id, &arvore.processo)?;
        if lidos.insert(id.clone(), destino).is_some() {
            return Err(repetidos);
        }
    }
    for movido in &mut arvore.documentos_movidos {
        movido.destino = lidos.remove(&movido.id).flatten();
    }
    Ok(())
}

fn destino_do_html(
    instalacao: &Instalacao,
    texto: &crate::tela_html::HtmlServido,
    id: &str,
    processo: &ProcessoNaArvore,
) -> Result<Option<DestinoDoMovido>, ErroArvore> {
    let dom = crate::tela_html::arvore(texto);
    let mut achado = None;
    for no in dom.elementos("a") {
        let Some(href) = no.atributo("href") else {
            continue;
        };
        if achado.is_some() {
            return Err(ErroArvore::DestinosDoDocumentoMovidoRepetidos);
        }
        achado = Some(conferir(instalacao, href, &texto_unico(no), id, processo)?);
    }
    Ok(achado)
}

fn conferir(
    instalacao: &Instalacao,
    href: &str,
    texto: &str,
    id: &str,
    processo: &ProcessoNaArvore,
) -> Result<DestinoDoMovido, ErroArvore> {
    let fora = || ErroArvore::DestinoDoDocumentoMovidoForaDaFormaObservada;
    let valores = |nome: &str| -> Vec<&str> {
        pares(href)
            .filter(|(chave, _)| *chave == nome)
            .map(|(_, valor)| valor)
            .collect()
    };
    let documento = documento_do_composto(id).ok_or_else(fora)?;
    let (destinos, documentos) = (valores("id_procedimento"), valores("id_documento"));
    let ([destino], [doc]) = (destinos.as_slice(), documentos.as_slice()) else {
        return Err(fora());
    };
    if *doc != documento
        || *destino == processo.id
        || !super::decimal(destino)
        || !valores("id_procedimento_anexado").is_empty()
    {
        return Err(fora());
    }
    let absoluta =
        conferir_endereco_relativo(instalacao, href, AcaoDoSei::ProcedimentoTrabalhar, destino)
            .map_err(ErroArvore::EnderecoDoDestinoMovidoRecusado)?;
    if texto.is_empty()
        || texto.trim() != texto
        || texto.contains(['<', '>', '&'])
        || texto == processo.nup
    {
        return Err(fora());
    }
    Ok(DestinoDoMovido {
        numero: texto.to_owned(),
        id_procedimento: (*destino).to_owned(),
        trabalhar: EnderecoDaArvore::conferido(absoluta),
    })
}
