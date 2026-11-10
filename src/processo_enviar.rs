//! `ies processo enviar NUP --para SIGLA [--manter-aberto] [--confirmar]`
//! (contracts/cli/processo-enviar.md): abre o processo como `processo
//! concluir`, lê o link de enviar da barra, pede a tela "Enviar Processo"
//! e a consulta do autocomplete que ela declara, e escolhe o destino que o
//! SEI publica. Prévia sem POST de escrita; com `--confirmar`, um POST e
//! uma leitura nova da árvore para verificar. Nunca repete a escrita.
mod anotacoes;
mod destino;
mod erro;
mod fluxo;
mod tela;

#[cfg(test)]
mod acervo;
#[cfg(test)]
mod acervo_busca;
#[cfg(test)]
mod tests_anotacoes;
#[cfg(test)]
mod tests_anti_ies;
#[cfg(test)]
pub(crate) mod tests_apoio;
#[cfg(test)]
mod tests_busca;
#[cfg(test)]
mod tests_destino;
#[cfg(test)]
mod tests_fallback;
#[cfg(test)]
mod tests_fluxo;
#[cfg(test)]
mod tests_reconciliacao;
#[cfg(test)]
mod tests_tela;

pub(crate) use anotacoes::texto_anotacao;
pub(crate) use erro::ErroEnviar;
pub(crate) use fluxo::enviar_processo;

use crate::processo::NumeroProcesso;
use crate::processo_concluir::FalhaDoEnvio;

/// O NUP, a `--unidade`, o destino pedido e as opções.
pub(crate) struct PedidoDoEnvio<'a> {
    pub(crate) nup: &'a NumeroProcesso,
    pub(crate) unidade: Option<&'a str>,
    /// A sigla de `--para`, já conferida pela CLI.
    pub(crate) para: &'a str,
    pub(crate) manter_aberto: bool,
    pub(crate) remover_anotacoes: bool,
    pub(crate) confirmar: bool,
}

/// O item do autocomplete escolhido como destino.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Destino {
    /// O texto de `descricao` antes de ` - `.
    pub(crate) sigla: String,
    /// `descricao` como servida (`SIGLA - Nome`).
    pub(crate) descricao: String,
    /// `id` do item, opaco.
    pub(crate) id_unidade: String,
}

/// Estratégia publicada pela instalação para remover a anotação.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum MetodoRemocao {
    /// Checkbox da própria tela de envio.
    CheckboxNoEnvio,
    /// Salvar a tela de anotação com descrição vazia antes da tramitação.
    LimparAnotacao,
}

/// Sucesso: prévia, ou enviado e conferido.
#[expect(
    clippy::struct_excessive_bools,
    reason = "estado do envio e duas opções independentes do formulário observado"
)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoEnviado {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    /// A unidade da sessão (`lnkInfraUnidade` da tela do processo).
    pub(crate) unidade: String,
    pub(crate) confirmado: bool,
    /// O POST saiu e a leitura confirmou o destino (e a origem como pedido).
    pub(crate) alterado: bool,
    pub(crate) manter_aberto: bool,
    pub(crate) remover_anotacoes: bool,
    /// Presença da anotação da unidade de origem, quando a tela a publicou.
    /// None significa não observada, nunca ausência.
    pub(crate) anotacao_antes: Option<bool>,
    /// Estado comprovado por um GET independente após a tramitação.
    pub(crate) anotacao_depois: Option<bool>,
    pub(crate) metodo_remocao: Option<MetodoRemocao>,
    pub(crate) destino: Destino,
    /// Siglas de `aberto_em` antes.
    pub(crate) aberto_em: Vec<String>,
    /// Siglas lidas na verificação (só depois da escrita).
    pub(crate) depois: Option<Vec<String>>,
    /// A resposta não foi a esperada e a leitura mostrou o envio.
    pub(crate) reconciliado_por: Option<FalhaDoEnvio>,
    pub(crate) requisicoes: u8,
}

impl ProcessoEnviado {
    /// `aberto_em` com o destino e, sem `manter_aberto`, sem a unidade da
    /// sessão: o que a verificação espera.
    pub(crate) fn depois_esperado(&self) -> Vec<String> {
        let mut esperado: Vec<String> = self
            .aberto_em
            .iter()
            .filter(|sigla| self.manter_aberto || **sigla != self.unidade)
            .cloned()
            .collect();
        esperado.push(self.destino.sigla.clone());
        esperado
    }
}
