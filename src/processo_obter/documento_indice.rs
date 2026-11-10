//! `documento DOC` pelo índice e por `--processo NUP` (Refs #60, fatia 2;
//! contracts/cli/documento-obter-web.md).
//!
//! - **Índice** (número SEI → processo `P` e documento `M`, aprendidos de
//!   toda árvore lida): link da árvore de `P` guardado nesta geração (2
//!   requisições), link `procedimento_trabalhar` guardado (3) e a entrada
//!   sem assinatura `procedimento_trabalhar&id_procedimento=P&id_documento=M`
//!   da lista branca (4). Palpite desmentido é esquecido e o comando cai
//!   **uma vez** na pesquisa rápida; nada se repete.
//! - **`--processo NUP`**: a abertura de `processo NUP` e o documento na
//!   árvore pelo número (a15), com as pastas abertas só se ele não estiver
//!   à vista.
use super::abertura::{ArvoreAberta, Contexto, Lida, Rede, fora_de_palpite};
use super::documento::{ErroDocumentoObter, FimDaLeitura, NoLido, localizar};
use super::fluxo::{abrir_processo, link_da_arvore, link_de_trabalho};
use super::login::{DestinoDoRedirecionamento, destino};
use super::{EnvioDoProcesso, ErroProcessoObter, ResolvidoPor, SessaoDoProcesso};
use crate::arvore_processo::{ErroTelaDoProcesso, ler_tela_do_documento};
use crate::erro_html::ErroHtml;
use crate::indice_local::{DocumentoGuardado, url_da_entrada_por_id_com_documento};
use crate::pesquisa_web::redirecionamento::{
    ErroRedirecionamentoDaPesquisa as Redir, OrigemDoTrabalho, conferir_trabalho,
};

type Aberto = (ArvoreAberta, Option<String>, ResolvidoPor);

/// `Ok(None)`: sem palpite no índice, ou palpite desmentido (esquecido).
pub(super) fn pelo_indice<V: EnvioDoProcesso, F: FimDaLeitura>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    numero: &str,
    leitura: F,
) -> Result<Option<F::Lido>, ErroDocumentoObter> {
    let (indice, instalacao) = (contexto.indice, &contexto.instalacao);
    let Some(guardado) = indice.documento_por_numero(instalacao, numero) else {
        return Ok(None);
    };
    let Some(nup) = indice.nup_por_id(instalacao, &guardado.id_procedimento) else {
        return Ok(None);
    };
    let processo = contexto.com_nup(&nup);
    let aberto = match link_da_arvore(&processo, rede)? {
        Some(aberto) => Some(aberto),
        None => match link_de_trabalho(&processo, rede)? {
            Some(aberto) => Some(aberto),
            None => entrada_com_documento(&processo, rede, &guardado, numero)?,
        },
    };
    let Some((arvore_aberta, unidade, resolvido_por)) = aberto else {
        return Ok(None);
    };
    // A árvore contraditória invalida o palpite mesmo quando localizar
    // recusa o número repetido. Nenhuma URL de conteúdo será consultada.
    processo.aprender_documentos(&arvore_aberta.arvore);
    let no = localizar(&arvore_aberta.arvore.documentos, numero)?;
    let Some(no) = no else {
        indice.esquecer_documento(instalacao, numero);
        return Ok(None);
    };
    let lido = NoLido {
        html: &arvore_aberta.html,
        no,
        processo: &arvore_aberta.arvore.processo,
        unidade,
        resolvido_por,
    };
    leitura.concluir(instalacao, rede, lido, numero).map(Some)
}

/// `--processo NUP`: o processo pelo caminho mais barato de `processo NUP`
/// e o único documento da árvore com o número pedido.
pub(super) fn pelo_processo<V: EnvioDoProcesso, F: FimDaLeitura>(
    contexto: &Contexto<'_>,
    sessao: &SessaoDoProcesso<'_>,
    rede: &mut Rede<'_, V>,
    numero: &str,
    leitura: F,
) -> Result<F::Lido, ErroDocumentoObter> {
    let (aberta, unidade, resolvido_por) = abrir_processo(contexto, sessao, rede, false)?;
    let no = localizar(&aberta.arvore.documentos, numero)?.ok_or(
        ErroHtml::ausente("numero_sei_ausente_da_arvore_do_processo")
            .com_mensagem("o número SEI não está na árvore do processo informado"),
    )?;
    let lido = NoLido {
        html: &aberta.html,
        no,
        processo: &aberta.arvore.processo,
        unidade,
        resolvido_por,
    };
    leitura.concluir(&contexto.instalacao, rede, lido, numero)
}

/// A entrada sem assinatura com documento (ADR 0003, lista branca): 302
/// para a mesma ação assinada com os dois ids, tela que seleciona `M` no
/// processo `P`, árvore. Só onde nenhuma das formas foi recusada; o aceite
/// e a recusa característica desta forma são registrados à parte.
fn entrada_com_documento<V: EnvioDoProcesso>(
    contexto: &Contexto<'_>,
    rede: &mut Rede<'_, V>,
    guardado: &DocumentoGuardado,
    numero: &str,
) -> Result<Option<Aberto>, ErroProcessoObter> {
    let (indice, instalacao) = (contexto.indice, &contexto.instalacao);
    let (id, documento) = (&guardado.id_procedimento, &guardado.id_documento);
    let recusada = indice.entrada_por_id(instalacao) == Some(false)
        || indice.entrada_por_id_com_documento(instalacao) == Some(false);
    // Um passo anterior desmentiu o par NUP ↔ P: o palpite não vale mais.
    if recusada || indice.id_por_nup(instalacao, contexto.nup).as_ref() != Some(id) {
        return Ok(None);
    }
    let Some(url) = url_da_entrada_por_id_com_documento(instalacao, id, documento) else {
        return Ok(None);
    };
    let localizacao = match rede.get(&url)? {
        Lida::Redirecionamento(localizacao) => localizacao,
        Lida::Login => {
            return Err(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ));
        }
        Lida::Html { .. } => return Err(ErroProcessoObter::RespostaDaEntradaPorIdNaoReconhecida),
        Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
    };
    match destino(localizacao.as_deref()) {
        DestinoDoRedirecionamento::RecusaSemAssinatura => {
            // A recusa derrubou a sessão: registrar e parar, sem relogin.
            indice.registrar_entrada_por_id_com_documento(instalacao, false);
            return Err(ErroProcessoObter::EntradaPorIdRecusada);
        }
        DestinoDoRedirecionamento::Login => {
            return Err(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ));
        }
        DestinoDoRedirecionamento::Outro => {}
    }
    let origem = OrigemDoTrabalho::EntradaPorIdComDocumento { id, documento };
    let esquecer = || indice.esquecer_documento(instalacao, numero);
    let trabalho = match conferir_trabalho(localizacao.as_deref(), &url, instalacao, origem) {
        Ok(redirecionado) => redirecionado.trabalho,
        Err(
            Redir::IdProcedimentoDoRedirecionamentoDiferenteDoPedido
            | Redir::IdDocumentoDoRedirecionamentoDiferenteDoPedido,
        ) => {
            indice.registrar_entrada_por_id_com_documento(instalacao, true);
            esquecer();
            return Ok(None);
        }
        Err(erro) => return Err(ErroProcessoObter::Redirecionamento(erro)),
    };
    indice.registrar_entrada_por_id_com_documento(instalacao, true);
    let (html, mime) = match rede.get(trabalho.as_str())? {
        Lida::Html { html, mime } => (html, mime),
        Lida::Redirecionamento(local) => return Err(fora_de_palpite(local.as_deref())),
        Lida::Login => {
            return Err(ErroProcessoObter::SessaoEncerrada(
                crate::erro_publico::SessaoEncerrada,
            ));
        }
        Lida::Falha(falha) => return Err(ErroProcessoObter::Transporte(falha)),
    };
    let tela = match ler_tela_do_documento(instalacao, &html, documento, contexto.unidade_pedida) {
        Ok((tela, processo)) if processo == *id => tela,
        Ok(_) | Err(ErroTelaDoProcesso::EnderecoDaArvoreDeOutroDocumento) => {
            esquecer();
            return Ok(None);
        }
        Err(erro) => return Err(ErroProcessoObter::Tela(erro)),
    };
    match rede.abrir_da_tela(contexto, trabalho.as_str(), (&html, &mime), tela, id) {
        Ok((aberta, unidade)) => Ok(Some((
            aberta,
            Some(unidade),
            ResolvidoPor::IndiceEntradaDocumento,
        ))),
        Err(ErroProcessoObter::Arvore(erro)) if erro.diverge_do_pedido() => {
            indice.esquecer_processo(instalacao, contexto.nup);
            esquecer();
            Ok(None)
        }
        Err(erro) => Err(erro),
    }
}

#[cfg(test)]
mod tests {
    use crate::arvore_processo::tests::{NUP, arvore, no_documento, no_processo};
    use crate::documento::NumeroDocumento;
    use crate::erro_html::ErroHtml;
    use crate::processo_obter::tests_apoio::*;
    use crate::processo_obter::{
        ErroDocumentoObter, PedidoDoAnexo, PedidoDoDocumento, SessaoDoProcesso, obter_documento,
    };
    use crate::transporte::sessao::CookieDaSessao;

    #[test]
    fn bdd_60_05_indice_ambiguo_nao_preserva_palpite_antigo() {
        // Teste tecnico do BDD-60-05: leitura pelo indice com numeros duplicados.
        let amb = ambiente();
        let inst = instalacao();
        amb.indice.registrar_processo(&inst, NUP, ID).unwrap();
        amb.indice
            .registrar_documentos(&inst, ID, &[("0000101", "99")])
            .unwrap();
        let geracao = geracao();
        amb.indice
            .registrar_link(
                &inst,
                chave(&geracao, "procedimento_visualizar", ID),
                &url_arvore(ID),
            )
            .unwrap();
        let duplicado = arvore(&[
            no_processo(0, ID, NUP),
            no_documento(1, "21", ID, "0000101"),
            no_documento(2, "23", ID, "0000101"),
        ]);
        let numero = NumeroDocumento::novo("0000101").unwrap();
        let url = url_controle();
        let sessao = SessaoDoProcesso {
            url_do_controle: &url,
            origem: ORIGEM,
            cookie: &CookieDaSessao::ler(&amb.cookie),
        };
        let pedido = PedidoDoDocumento {
            numero: &numero,
            unidade: None,
            processo: None,
            anexo: PedidoDoAnexo::default(),
        };
        let mut rede = Roteiro::novo(vec![ok(duplicado)]);
        let resultado = obter_documento(&sessao, &pedido, &amb.indice, &mut rede);
        assert_eq!(
            resultado.unwrap_err(),
            ErroDocumentoObter::Html(ErroHtml::repetido("numero_sei_repetido_na_arvore"))
        );
        assert_eq!(rede.pedidos, [get(&url_arvore(ID))]);
        assert!(amb.indice.documento_por_numero(&inst, "0000101").is_none());
    }
}
