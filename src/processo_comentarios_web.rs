//! Leitura somente GET da lista de comentários com identidade do processo.
//! A fonte da URL é a barra da árvore validada, nunca um endereço sintetizado.
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_comentarios::{
    ComentarioLido, ErroComentarios, ErroLinkDosComentarios, TelaDosComentarios,
    VisaoDosComentarios,
};
use crate::processo_concluir::escrita::CookieIlegivel;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::FalhaTransporte;

pub(crate) struct Resultado {
    pub(crate) numero: String,
    pub(crate) id_procedimento: String,
    pub(crate) visao: VisaoDosComentarios,
    pub(crate) itens: Vec<ComentarioLido>,
    pub(crate) total_declarado: usize,
    pub(crate) requisicoes: u8,
}

#[derive(Debug, thiserror::Error)]
#[error("{self:?}")]
pub(crate) enum ErroProcessoComentarios {
    Abertura(#[from] ErroProcessoObter),
    Barra(#[from] ErroLinkDosComentarios),
    Cookie(#[from] CookieIlegivel),
    Transporte(#[from] FalhaTransporte),
    Leitura(#[from] ErroComentarios),
    ComentariosNaoOferecidos,
    RedirecionamentoNaoComprovado,
    SessaoEncerrada(#[source] crate::erro_publico::SessaoEncerrada),
}

/// Exige uma lista completa, sem supor lista vazia quando a URL está ausente.
///
/// # Errors
/// Recusa link não publicado, resposta diferente da lista completa e falhas da fonte.
pub(crate) fn ler<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Resultado, ErroProcessoComentarios> {
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, false), indice, &mut rede)?;
    let endereco = acoes
        .comentarios?
        .ok_or(ErroProcessoComentarios::ComentariosNaoOferecidos)?;
    let cookie = ler_cookie(sessao)?;
    let (html, _) =
        obter_tela(&mut rede, &cookie, endereco.as_str()).map_err(|falha| match falha {
            FalhaDaTela::Transporte(erro) => ErroProcessoComentarios::Transporte(erro),
            FalhaDaTela::Redirecionada => ErroProcessoComentarios::RedirecionamentoNaoComprovado,
            FalhaDaTela::SessaoEncerrada(crate::erro_publico::SessaoEncerrada) => {
                ErroProcessoComentarios::SessaoEncerrada(crate::erro_publico::SessaoEncerrada)
            }
        })?;
    // O cadastro vazio não pode ser confundido com uma lista completa:
    // a transição 302/lista vazia receberá tratamento próprio em outra fatia.
    let TelaDosComentarios::Lista(lista) = crate::processo_comentarios::ler_tela(&html)? else {
        return Err(ErroProcessoComentarios::RedirecionamentoNaoComprovado);
    };
    Ok(Resultado {
        numero: nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id,
        visao: lista.visao,
        itens: lista.itens,
        total_declarado: lista.total_declarado,
        requisicoes: rede.feitas(),
    })
}
