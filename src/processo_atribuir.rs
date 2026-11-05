//! Atribuição de processo: leitura estrutural da tela e escolha estrita.
//! Observado em UnB e GO (cassettes processo-atribuir); oráculo:
//! Todos tests/test_ciclo_atribuir_010.py @323eb01.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct UsuarioDaAtribuicao {
    pub(crate) id: String,
    pub(crate) login: String,
    pub(crate) rotulo: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct TelaDaAtribuicao {
    pub(crate) usuarios: Vec<UsuarioDaAtribuicao>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroAtribuir {
    TelaForaDaForma,
    UsuarioNaoAtribuivel,
    UsuarioAmbiguo,
    SituacaoNaoObservada,
    AtribuicaoJaAplicada,
}
impl ErroAtribuir {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::TelaForaDaForma => "tela_da_atribuicao_fora_da_forma",
            Self::UsuarioNaoAtribuivel => "usuario_nao_atribuivel",
            Self::UsuarioAmbiguo => "usuario_ambiguo_na_atribuicao",
            Self::SituacaoNaoObservada => "situacao_da_atribuicao_nao_observada",
            Self::AtribuicaoJaAplicada => "atribuicao_ja_aplicada",
        }
    }
}
pub(crate) fn ler_tela(_html: &str, _id: &str) -> Result<TelaDaAtribuicao, ErroAtribuir> {
    Err(ErroAtribuir::TelaForaDaForma)
}
pub(crate) fn planejar(
    _tela: &TelaDaAtribuicao,
    _login: Option<&str>,
    _atual: Option<&str>,
) -> Result<String, ErroAtribuir> {
    Err(ErroAtribuir::UsuarioNaoAtribuivel)
}

#[cfg(test)]
#[path = "processo_atribuir/tests.rs"]
mod tests;
