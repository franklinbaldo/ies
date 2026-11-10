//! Prévia de anotação da unidade: apenas GET do formulário servido.
//! Os POSTs de registro/remoção pertencem às próximas fatias BDD-ANOT.
use crate::arvore_processo::MotivoAbertoEm;
use crate::erro_html::ErroHtml;
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_concluir::escrita::{
    Contador, FalhaDaTela, leitura, ler_cookie, obter_tela, siglas_abertas,
};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever,
};
use crate::transporte::FalhaTransporte;

pub(crate) struct Previa {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) texto: String,
    pub(crate) anotacao_antes: String,
    pub(crate) requisicoes: u8,
}

/// Uma variante por causa; o diagnóstico é derivado do tipo Rust.
#[derive(Debug, crate::erro_publico::Causa)]
pub(crate) enum Erro {
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    #[causa(interna)]
    Barra(ErroHtml),
    #[causa(interna)]
    Leitura(ErroHtml),
    #[causa(interna)]
    Transporte(FalhaTransporte),
    #[causa(interna)]
    Cookie(FalhaTransporte),
    TelaRedirecionada,
    #[causa(sessao_encerrada)]
    SessaoEncerrada,
    UnidadeNaoPublicada,
    SituacaoNaoLida(MotivoAbertoEm),
    ProcessoForaDaUnidade,
    AnotacaoNaoOferecida,
}

/// Uma prévia não altera a anotação, ainda que a tela ofereça o botão Salvar.
pub(crate) fn prever<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    nup: &NumeroProcesso,
    texto: &str,
    unidade: Option<&str>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<Previa, Erro> {
    let mut rede = Contador::novo(envio);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &leitura(nup, unidade, true), indice, &mut rede)
            .map_err(Erro::Abertura)?;
    let unidade = aberto.unidade.ok_or(Erro::UnidadeNaoPublicada)?;
    let abertas = siglas_abertas(&aberto.arvore.aberto_em).map_err(Erro::SituacaoNaoLida)?;
    if !abertas.contains(&unidade) {
        return Err(Erro::ProcessoForaDaUnidade);
    }
    let id = aberto.arvore.processo.id;
    let url = acoes
        .anotacoes
        .map_err(Erro::Barra)?
        .ok_or(Erro::AnotacaoNaoOferecida)?;
    let cookie = ler_cookie(sessao).map_err(Erro::Cookie)?;
    let (html, _) = obter_tela(&mut rede, &cookie, &url).map_err(|falha| match falha {
        FalhaDaTela::Transporte(erro) => Erro::Transporte(erro),
        FalhaDaTela::Redirecionada => Erro::TelaRedirecionada,
        FalhaDaTela::SessaoEncerrada => Erro::SessaoEncerrada,
    })?;
    let anotacao_antes =
        crate::processo_enviar::texto_anotacao(&html, &id).map_err(Erro::Leitura)?;
    Ok(Previa {
        nup: nup.as_str().to_owned(),
        id_procedimento: id,
        unidade,
        texto: texto.to_owned(),
        anotacao_antes,
        requisicoes: rede.feitas(),
    })
}
