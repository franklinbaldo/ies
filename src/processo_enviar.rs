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
mod releitura_anotacao;
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
mod tests_causa_releitura;
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

pub(crate) use anotacoes::{
    ErroLeituraDaAnotacao, REGISTRAR_ANOTACAO, salvar_oferecido, texto_anotacao,
};
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

/// A remoção da anotação só existe quando foi pedida: o método publicado e
/// o que as leituras viram. `antes` e `depois` None significam não observada.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RemocaoDaAnotacao {
    pub(crate) metodo: MetodoRemocao,
    /// Presença da anotação da unidade de origem, quando a tela a publicou.
    /// None significa não observada, nunca ausência.
    pub(crate) antes: Option<bool>,
    /// Presença comprovada por um GET independente, após a limpeza ou o envio.
    pub(crate) depois: Option<bool>,
}

/// O desfecho do envio: prévia sem POST, ou tramitado e conferido por leitura.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DesfechoDoEnvio {
    /// Sem `--confirmar`: nenhum POST de escrita saiu.
    Previa,
    /// O POST saiu e a leitura confirmou o destino (e a origem como pedido).
    Enviado {
        /// Siglas lidas na verificação (só depois da escrita).
        depois: Vec<String>,
        /// A resposta não foi a esperada e a leitura mostrou o envio.
        reconciliado_por: Option<FalhaDoEnvio>,
    },
}

/// Sucesso: prévia, ou enviado e conferido.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoEnviado {
    pub(crate) nup: String,
    pub(crate) id_procedimento: String,
    /// A unidade da sessão (`lnkInfraUnidade` da tela do processo).
    pub(crate) unidade: String,
    pub(crate) manter_aberto: bool,
    pub(crate) remocao: Option<RemocaoDaAnotacao>,
    pub(crate) destino: Destino,
    /// Siglas de `aberto_em` antes.
    pub(crate) aberto_em: Vec<String>,
    pub(crate) desfecho: DesfechoDoEnvio,
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

    /// Só o envio confirmado pede POST: a prévia nunca é confirmada.
    pub(crate) fn confirmado(&self) -> bool {
        self.alterado()
    }

    /// O POST saiu e a leitura confirmou o destino.
    pub(crate) fn alterado(&self) -> bool {
        matches!(self.desfecho, DesfechoDoEnvio::Enviado { .. })
    }

    pub(crate) fn depois(&self) -> Option<&[String]> {
        match &self.desfecho {
            DesfechoDoEnvio::Previa => None,
            DesfechoDoEnvio::Enviado { depois, .. } => Some(depois.as_slice()),
        }
    }

    pub(crate) fn reconciliado_por(&self) -> Option<&FalhaDoEnvio> {
        match &self.desfecho {
            DesfechoDoEnvio::Previa => None,
            DesfechoDoEnvio::Enviado {
                reconciliado_por, ..
            } => reconciliado_por.as_ref(),
        }
    }

    pub(crate) fn remover_anotacoes(&self) -> bool {
        self.remocao.is_some()
    }

    pub(crate) fn metodo_remocao(&self) -> Option<MetodoRemocao> {
        self.remocao.as_ref().map(|remocao| remocao.metodo)
    }

    pub(crate) fn anotacao_antes(&self) -> Option<bool> {
        self.remocao.as_ref().and_then(|remocao| remocao.antes)
    }

    pub(crate) fn anotacao_depois(&self) -> Option<bool> {
        self.remocao.as_ref().and_then(|remocao| remocao.depois)
    }

    /// Registra a presença da anotação observada por um GET. Sem remoção
    /// pedida não há o que registrar.
    pub(crate) fn com_anotacao_depois(self, presente: bool) -> Self {
        let remocao = self.remocao.map(|remocao| RemocaoDaAnotacao {
            depois: Some(presente),
            ..remocao
        });
        Self { remocao, ..self }
    }

    /// O envio conferido: a leitura de verificação e a falha da resposta.
    pub(crate) fn enviado(
        self,
        depois: Vec<String>,
        reconciliado_por: Option<FalhaDoEnvio>,
    ) -> Self {
        Self {
            desfecho: DesfechoDoEnvio::Enviado {
                depois,
                reconciliado_por,
            },
            ..self
        }
    }
}
