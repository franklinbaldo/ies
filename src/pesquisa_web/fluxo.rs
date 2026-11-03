//! `ies processo buscar TERMO [--pagina N]` (contracts/cli/processo-buscar.md):
//! GET do Controle → link do menu `protocolo_pesquisar` → GET da tela →
//! POST da pesquisa → (N > 1) POST de `navegar(inicio)`. Cada chamada do
//! envio é uma requisição real única, sem retry nem redirect seguido; a
//! sessão fica na página/filtro em que a pesquisa a deixou.
use super::erro_busca::ErroBuscaDeProcesso;
use super::resposta::RespostaDaPesquisa;
use crate::processo::NumeroProcesso;
use crate::visao_do_controle::EnvioDoControle;
use std::path::Path;

/// Arquivos privados da sessão já lidos: URL assinada do Controle e origem.
pub(crate) struct SessaoDaBusca<'a> {
    pub(crate) url_do_controle: &'a str,
    pub(crate) origem: &'a str,
    pub(crate) cookie_file: &'a Path,
}

/// O que o usuário pediu: termo, página opcional e `--unidade` conferida.
pub(crate) struct PedidoDeBusca<'a> {
    pub(crate) termo: &'a NumeroProcesso,
    pub(crate) pagina: Option<u32>,
    pub(crate) unidade: Option<&'a str>,
}

/// Resposta observada, unidade do Controle e requisições feitas.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BuscaRealizada {
    pub(crate) unidade: String,
    pub(crate) resposta: RespostaDaPesquisa,
    pub(crate) requisicoes: u8,
}

/// Orçamento: página 1 = GET + GET + POST; página N = mais um POST.
pub(crate) fn buscar_processo<V: EnvioDoControle>(
    _sessao: &SessaoDaBusca<'_>,
    _pedido: &PedidoDeBusca<'_>,
    _envio: &mut V,
) -> Result<BuscaRealizada, ErroBuscaDeProcesso> {
    todo!("GREEN #58: fluxo da busca")
}

#[cfg(test)]
#[path = "tests_fluxo.rs"]
mod tests;
