//! Prévia Web injetável da atribuição de processo: abertura, unidade,
//! atribuição publicada na árvore, formulário da barra e payload local.
//! A CLI mantém --confirmar fechado até haver validação live em treinamento.
use crate::arvore_processo::{AbertoEm, ErroAtribuirNaBarra};
use crate::formulario_web::{ErroSubmissaoFormulario, enviar_submissao_via};
use crate::indice_local::IndiceLocal;
use crate::instalacao::Instalacao;
use crate::processo::NumeroProcesso;
use crate::processo_atribuir::{ErroAtribuir, preparar_submissao};
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, leitura, ler_cookie, obter_tela};
use crate::processo_obter::{
    EnvioDoProcesso, ErroProcessoObter, SessaoDoProcesso, abrir_para_escrever, obter_processo,
};
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};
use crate::visao_do_controle::EnvioDoControle as _;

pub(crate) struct PedidoDaAtribuicao<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    pub(crate) usuario: Option<&'a str>,
    pub(crate) confirmar: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoAtribuido {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    pub(crate) unidade: String,
    pub(crate) usuario_atual: String,
    pub(crate) usuario_pedido: Option<String>,
    pub(crate) valor_da_opcao: String,
    pub(crate) confirmado: bool,
    pub(crate) alterado: bool,
    pub(crate) reconciliado_por: Option<ErroSubmissaoFormulario>,
    pub(crate) requisicoes: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, crate::erro_publico::Causa)]
pub(crate) enum ErroProcessoAtribuir {
    #[causa(interna)]
    Abertura(ErroProcessoObter),
    UnidadeDaSessaoNaoPublicada,
    SituacaoDaAtribuicaoNaoObservada,
    AtribuicaoNaoOferecidaNaBarra,
    #[causa(interna)]
    Barra(ErroAtribuirNaBarra),
    #[causa(interna)]
    Cookie(FalhaTransporte),
    #[causa(interna)]
    Tela(FalhaTransporte),
    TelaDaAtribuicaoRedirecionada,
    SessaoEncerradaNaAtribuicao,
    OrigemNaoAutorizada,
    #[causa(nome = "escrita_de_atribuicao_ainda_nao_verificada")]
    EscritaAindaNaoVerificada,
    #[causa(interna)]
    Preparacao(ErroAtribuir),
    #[causa(interna)]
    Formulario(ErroSubmissaoFormulario),
    #[causa(nome = "efeito_incerto_na_atribuicao")]
    EfeitoIncertoNaAtribuicao {
        falha_do_envio: Option<ErroSubmissaoFormulario>,
        leitura: Option<ErroProcessoObter>,
        atribuicao_relida: Option<String>,
    },
}

impl ErroProcessoAtribuir {
    pub(crate) fn subcodigo(&self) -> &'static str {
        match self {
            Self::Preparacao(erro) => erro.clone().subcodigo(),
            Self::Barra(erro) => (*erro).subcodigo(),
            _ => crate::erro_publico::folha(self),
        }
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        match self {
            Self::Formulario(erro) => erro.status_http(),
            Self::EfeitoIncertoNaAtribuicao { falha_do_envio, .. } => {
                falha_do_envio.as_ref().and_then(ErroSubmissaoFormulario::status_http)
            }
            _ => crate::erro_publico::Causa::status_http(self),
        }
    }

    pub(crate) fn saida(&self) -> u8 {
        if matches!(self, Self::EfeitoIncertoNaAtribuicao { .. }) { 3 } else { 1 }
    }

    pub(crate) fn mensagem(&self) -> &'static str {
        match self {
            Self::Abertura(erro) => erro.mensagem(),
            Self::UnidadeDaSessaoNaoPublicada => "a abertura não publicou a unidade da sessão",
            Self::SituacaoDaAtribuicaoNaoObservada => {
                "a árvore não publicou a atribuição na unidade da sessão"
            }
            Self::AtribuicaoNaoOferecidaNaBarra => {
                "a barra do processo não oferece atribuição nesta unidade"
            }
            Self::Barra(_) => "o link de atribuição não foi publicado na forma observada",
            Self::Cookie(_) => "o arquivo privado não forneceu um cookie válido",
            Self::Tela(_) => "a tela de atribuição não foi servida",
            Self::TelaDaAtribuicaoRedirecionada => {
                "a tela de atribuição redirecionou; nenhum formulário foi enviado"
            }
            Self::SessaoEncerradaNaAtribuicao => {
                "a sessão encerrou antes da atribuição; rode ies login"
            }
            Self::OrigemNaoAutorizada => "a origem da sessão não é autorizada",
            Self::EscritaAindaNaoVerificada => {
                "a atribuição confirmada ainda não tem releitura de prova"
            }
            Self::Preparacao(_) => "o formulário de atribuição não corresponde ao observado",
            Self::Formulario(_) => "a submissão da atribuição não pôde ser enviada",
            Self::EfeitoIncertoNaAtribuicao { .. } => {
                "a releitura independente não comprovou a atribuição; confira antes de repetir"
            },
        }
    }
}

/// A primeira requisição só ocorre depois da conferência da trava de escrita.
pub(crate) fn executar_previa<V: EnvioDoProcesso>(
    sessao: &SessaoDoProcesso<'_>,
    pedido: &PedidoDaAtribuicao<'_>,
    indice: &IndiceLocal,
    envio: &mut V,
) -> Result<ProcessoAtribuido, ErroProcessoAtribuir> {
    use ErroProcessoAtribuir as E;
    let mut rede = Contador::novo(envio);
    let abertura = leitura(pedido.nup, pedido.unidade, true);
    let (aberto, acoes) =
        abrir_para_escrever(sessao, &abertura, indice, &mut rede).map_err(E::Abertura)?;
    let unidade = aberto
        .unidade
        .as_deref()
        .ok_or(E::UnidadeDaSessaoNaoPublicada)?;
    let atual = atribuicao_observada(&aberto.arvore.aberto_em, unidade)
        .ok_or(E::SituacaoDaAtribuicaoNaoObservada)?;
    let link = acoes
        .atribuir
        .map_err(E::Barra)?
        .ok_or(E::AtribuicaoNaoOferecidaNaBarra)?;
    let cookie = ler_cookie(sessao).map_err(E::Cookie)?;
    let (html, mime) =
        obter_tela(&mut rede, &cookie, link.as_str()).map_err(|erro| match erro {
            FalhaDaTela::Transporte(falha) => E::Tela(falha),
            FalhaDaTela::Redirecionada => E::TelaDaAtribuicaoRedirecionada,
            FalhaDaTela::SessaoEncerrada => E::SessaoEncerradaNaAtribuicao,
        })?;
    let instalacao = Instalacao::interpretar(sessao.origem).ok_or(E::OrigemNaoAutorizada)?;
    let (valor, submissao) = preparar_submissao(
        &instalacao,
        link.as_str(),
        &html,
        &mime,
        &aberto.arvore.processo.id,
        unidade,
        pedido.usuario,
        Some(atual),
    )
    .map_err(E::Preparacao)?;
    Ok(ProcessoAtribuido {
        nup: pedido.nup.as_str().to_owned(),
        id_procedimento: aberto.arvore.processo.id,
        unidade: unidade.to_owned(),
        usuario_atual: atual.to_owned(),
        usuario_pedido: pedido.usuario.map(str::to_owned),
        valor_da_opcao: valor,
        confirmado: false,
        alterado: false,
        requisicoes: rede.feitas(),
    })
}
