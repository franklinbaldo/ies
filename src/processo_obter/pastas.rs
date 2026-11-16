//! Pastas fechadas depois da árvore (contracts/cli/processo-obter-pastas.md):
//! com algum `AGUARDE`, um GET de "Abrir todas as Pastas" e a releitura da
//! árvore inteira; o link conferido é guardado no índice com objeto
//! `<id>:abrir_pastas`, tentado primeiro no caminho 1. Nunca
//! `FECHAR_PASTAS`, nunca restauração.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite, ler_arvore_da_pagina};
use super::{EnvioDoProcesso, ErroProcessoObter};
use crate::acao_do_sei::AcaoDoSei;
use crate::arvore_processo::{ErroArvore, ler_acao_de_abrir_pastas};
use crate::formulario_web::{EfeitoAvaliado, EstadoDeSessao};
use crate::instalacao::Instalacao;

/// Efeito do GET de `ABRIR_PASTAS`: a ação irmã `FECHAR_PASTAS` sugere que
/// o servidor lembra as pastas abertas (não medido pelo IES; nas cassettes
/// da `UnB` a árvore seguinte voltou fechada). Não é escrita.
const EFEITO_DE_ABRIR_PASTAS: EfeitoAvaliado = EfeitoAvaliado::AlteraEstadoDaSessao {
    o_que: EstadoDeSessao::PaginaOuFiltroLembrado,
};
const _: () = assert!(matches!(
    EFEITO_DE_ABRIR_PASTAS,
    EfeitoAvaliado::AlteraEstadoDaSessao { .. }
));

/// Objeto do link de abrir pastas no índice, distinto do da árvore (`<id>`).
pub(super) fn objeto(id: &str) -> String {
    format!("{id}:abrir_pastas")
}

/// A árvore já conferida, completada se tem pasta fechada (e o documento
/// procurado, se houver, não está à vista). Depois da
/// primeira árvore a identidade está provada: tudo aqui é erro, não palpite.
pub(super) fn completar<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    html: &str,
    mut aberta: ArvoreAberta,
) -> Result<ArvoreAberta, ErroProcessoObter> {
    if !aberta.arvore.documentos_incluidos {
        return Ok(aberta);
    }
    let procurado_visivel = contexto.documento_procurado.is_some_and(|numero| {
        aberta
            .arvore
            .documentos
            .iter()
            .any(|d| d.numero_sei.as_deref() == Some(numero))
    });
    if !aberta.arvore.pastas_nao_carregadas || procurado_visivel {
        return Ok(aberta);
    }
    let id = aberta.arvore.processo.id.clone();
    let Some(endereco) = ler_acao_de_abrir_pastas(&contexto.instalacao, html, &id)? else {
        if contexto.permitir_leitura_parcial {
            aberta.arvore.documentos_incluidos = false;
            aberta.arvore.documentos.clear();
            aberta.arvore.documentos_movidos.clear();
            aberta.arvore.pastas.clear();
            aberta.arvore.aviso_leitura_parcial =
                Some(ErroArvore::PastasNaoCarregadasSemAcaoDeAbrir);
            return Ok(aberta);
        }
        return Err(ErroProcessoObter::Arvore(
            ErroArvore::PastasNaoCarregadasSemAcaoDeAbrir,
        ));
    };
    let html = match rede.get(endereco.as_str())? {
        Lida::Html { html, .. } => html,
        Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
        Lida::Login => {
            return Err(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ));
        }
        Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
    };
    let completa = match aberta_inteira(contexto, &html, &id) {
        Ok(c) => c,
        Err(erro) => {
            if contexto.permitir_leitura_parcial && erro.admite_leitura_parcial() {
                aberta.arvore.documentos_incluidos = false;
                aberta.arvore.documentos.clear();
                aberta.arvore.documentos_movidos.clear();
                aberta.arvore.pastas.clear();
                aberta.arvore.aviso_leitura_parcial = Some(erro);
                return Ok(aberta);
            }
            return Err(ErroProcessoObter::Arvore(erro));
        }
    };
    contexto.aprender_link(
        AcaoDoSei::ProcedimentoVisualizar,
        &objeto(&id),
        endereco.as_str(),
    );
    Ok(completa)
}

/// A árvore servida por "Abrir todas as Pastas": conferida e sem `AGUARDE`.
pub(super) fn aberta_inteira(
    contexto: &Contexto<'_>,
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<ArvoreAberta, ErroArvore> {
    aberta_inteira_da_pagina(&contexto.instalacao, contexto.nup, html, id)
}

/// A mesma conferência, sem rede nem índice (também a do replay do acervo).
pub(super) fn aberta_inteira_da_pagina(
    instalacao: &Instalacao,
    nup: &str,
    html: &crate::tela_html::HtmlServido,
    id: &str,
) -> Result<ArvoreAberta, ErroArvore> {
    let completa = ler_arvore_da_pagina(instalacao, nup, html, id)?;
    if completa.arvore.pastas_nao_carregadas {
        return Err(ErroArvore::PastasNaoCarregadasAposAbrirPastas);
    }
    Ok(completa)
}

/// O link de abrir pastas guardado, tentado antes do link da árvore.
pub(super) enum Guardado {
    Aberta(Box<ArvoreAberta>),
    /// Não havia link, ou ele envelheceu e foi esquecido: segue o da árvore.
    Seguir,
    /// A árvore é de outro processo: o par NUP ↔ id foi esquecido.
    Divergiu,
}

pub(super) fn pelo_link_guardado<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    id: &str,
) -> Result<Guardado, ErroProcessoObter> {
    let objeto = objeto(id);
    let instalacao = &contexto.instalacao;
    let Some(link) = contexto
        .chave(AcaoDoSei::ProcedimentoVisualizar, &objeto)
        .and_then(|chave| contexto.indice.link(instalacao, chave))
    else {
        return Ok(Guardado::Seguir);
    };
    if let Lida::Html { html, .. } = rede.get(link.url())? {
        match aberta_inteira(contexto, &html, id) {
            Ok(aberta) => {
                contexto
                    .indice
                    .registrar_processo(instalacao, contexto.nup, id);
                contexto.aprender_link(AcaoDoSei::ProcedimentoVisualizar, &objeto, link.url());
                return Ok(Guardado::Aberta(Box::new(aberta)));
            }
            Err(erro) if erro.diverge_do_pedido() => {
                contexto.indice.esquecer_processo(instalacao, contexto.nup);
                contexto.esquecer_link(AcaoDoSei::ProcedimentoVisualizar, &objeto);
                return Ok(Guardado::Divergiu);
            }
            Err(ErroArvore::NoDoProcessoAusenteNaArvore) => {}
            Err(erro) => return Err(ErroProcessoObter::Arvore(erro)),
        }
    }
    contexto.esquecer_link(AcaoDoSei::ProcedimentoVisualizar, &objeto);
    Ok(Guardado::Seguir)
}
