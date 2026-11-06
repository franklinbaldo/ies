//! Fluxo injetável de atribuição. RED: faltam abertura, prévia e prova por leitura.
use crate::indice_local::IndiceLocal;
use crate::processo::NumeroProcesso;
use crate::processo_obter::{EnvioDoProcesso, SessaoDoProcesso};
use crate::processo_atribuir::ErroAtribuir;
use crate::processo_obter::ErroProcessoObter;
use crate::arvore_processo::ErroAtribuirNaBarra;
use crate::transporte::{FalhaTransporte, StatusHttpRecusado};

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
    #[causa(interna)]
    Preparacao(ErroAtribuir),
}

impl ErroProcessoAtribuir {
    pub(crate) fn subcodigo(&self) -> &'static str {
        crate::erro_publico::folha(self)
    }

    pub(crate) fn status_http(&self) -> Option<&StatusHttpRecusado> {
        crate::erro_publico::Causa::status_http(self)
    }
}

/// Implementação intencionalmente vermelha: o teste deve falhar antes do GREEN.
pub(crate) fn executar_previa<V: EnvioDoProcesso>(
    _sessao: &SessaoDoProcesso<'_>,
    _pedido: &PedidoDaAtribuicao<'_>,
    _indice: &IndiceLocal,
    _envio: &mut V,
) -> Result<ProcessoAtribuido, ErroProcessoAtribuir> {
    Err(ErroProcessoAtribuir::SituacaoDaAtribuicaoNaoObservada)
}
