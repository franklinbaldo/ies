//! Leitura independente da anotação depois de escrever: o erro conserva a causa.
use super::anotacoes::{self, ErroDasAnotacoes};
use super::erro::ErroEnviar;
use crate::processo_concluir::escrita::{Contador, FalhaDaTela, obter_tela};
use crate::processo_obter::EnvioDoProcesso;

/// Mesmo GET e classificação utilizados antes e depois da escrita.
pub(super) fn obter_tela_das_anotacoes<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
) -> Result<(String, String), ErroEnviar> {
    obter_tela(rede, cookie, link).map_err(|falha| match falha {
        FalhaDaTela::Transporte(falha) => {
            ErroEnviar::from(ErroDasAnotacoes::TelaDasAnotacoes(falha))
        }
        FalhaDaTela::Redirecionada => {
            ErroEnviar::from(ErroDasAnotacoes::TelaDasAnotacoesRedirecionada)
        }
        FalhaDaTela::SessaoEncerrada(causa) => ErroEnviar::SessaoEncerrada(causa),
    })
}

/// Um GET novo retorna presença comprovada ou a causa tipada da falha.
pub(super) fn observar_anotacao_depois<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    id: &str,
) -> Result<bool, ErroEnviar> {
    let (html, _) = obter_tela_das_anotacoes(rede, cookie, link)?;
    anotacoes::anotacao_presente(&html, id)
        .map_err(|causa| ErroEnviar::from(ErroDasAnotacoes::from(causa)))
}

/// Limpeza só é comprovada por campo vazio do mesmo processo.
pub(super) fn conferir_anotacao_depois<V: EnvioDoProcesso>(
    rede: &mut Contador<'_, V>,
    cookie: &str,
    link: &str,
    id: &str,
    unidades_apos_envio: Option<&[String]>,
) -> Result<(), ErroEnviar> {
    match observar_anotacao_depois(rede, cookie, link, id) {
        Ok(false) => Ok(()),
        Ok(true) => Err(ErroEnviar::AnotacoesNaoRemovidasAposEnvio {
            depois: unidades_apos_envio.map(ToOwned::to_owned),
        }),
        Err(causa) => Err(ErroEnviar::NaoConfirmadoNaRemocaoDeAnotacoes {
            depois: unidades_apos_envio.map(ToOwned::to_owned),
            causa: Box::new(causa),
        }),
    }
}
