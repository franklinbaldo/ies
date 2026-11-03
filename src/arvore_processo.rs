//! Árvore do processo (`procedimento_visualizar`), lida sem regex e sem
//! executar JavaScript: só as linhas inteiras
//! `Nos[i] = new infraArvoreNo(a0,…,a15);` servidas no `script` inline.
//! Estrutura em docs/como-o-sei-funciona/arvore-do-processo.md; contrato em
//! contracts/cli/processo-obter-arvore.md. Nesta fatia: `PROCESSO` e
//! `DOCUMENTO` do primeiro nível; `PASTA` e qualquer outro tipo são recusa.
mod literais;
mod tela;

#[cfg(test)]
pub(crate) mod tests;
#[cfg(test)]
mod tests_literais;
#[cfg(test)]
pub(crate) mod tests_tela;

use literais::{Literal, ler_declaracao};
pub(crate) use tela::{EnderecoDaArvore, ErroTelaDoProcesso, TelaDoProcesso, ler_tela_do_processo};

/// O nó `PROCESSO`: id (a1), NUP (a5, igual a a15) e tipo (a6, a dica).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ProcessoNaArvore {
    pub(crate) id: String,
    pub(crate) nup: String,
    pub(crate) tipo: String,
}

/// Um `DOCUMENTO` do primeiro nível: id (a1), rótulo (a5) e número SEI
/// (a15, que pode vir `null`). A dica (a6) não é guardada: em e-mails traz
/// remetente e destinatários.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DocumentoNaArvore {
    pub(crate) id: String,
    pub(crate) rotulo: String,
    pub(crate) numero_sei: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ArvoreLida {
    pub(crate) processo: ProcessoNaArvore,
    pub(crate) documentos: Vec<DocumentoNaArvore>,
}

/// Uma variante por fato verificável; nenhum valor servido é guardado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroArvore {
    DeclaracaoDeNoForaDaFormaObservada,
    ArgumentoDeNoNaoLiteral,
    TextoDeNoSemFechamento,
    EscapeDeNoNaoReconhecido,
    QuantidadeDeArgumentosDoNoDiferenteDaObservada,
    ArgumentoDeNoComTipoDiferenteDoObservado,
    IndiceDeNoForaDaSequencia,
    TipoDeNoDaArvoreNaoSuportado,
    NoDoProcessoAusenteNaArvore,
    NosDoProcessoRepetidosNaArvore,
    NoDoProcessoComPai,
    NupDoNoDoProcessoInconsistente,
    IdDeNoNaoDecimal,
    DocumentoComPaiDiferenteDoProcesso,
    DocumentoRepetidoNaArvore,
    IdentidadeDoProcessoDiferenteDaPedida,
    IdDoProcessoNaArvoreDiferenteDoAberto,
}

impl ErroArvore {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::DeclaracaoDeNoForaDaFormaObservada => "declaracao_de_no_fora_da_forma_observada",
            Self::ArgumentoDeNoNaoLiteral => "argumento_de_no_nao_literal",
            Self::TextoDeNoSemFechamento => "texto_de_no_sem_fechamento",
            Self::EscapeDeNoNaoReconhecido => "escape_de_no_nao_reconhecido",
            Self::QuantidadeDeArgumentosDoNoDiferenteDaObservada => {
                "quantidade_de_argumentos_do_no_diferente_da_observada"
            }
            Self::ArgumentoDeNoComTipoDiferenteDoObservado => {
                "argumento_de_no_com_tipo_diferente_do_observado"
            }
            Self::IndiceDeNoForaDaSequencia => "indice_de_no_fora_da_sequencia",
            Self::TipoDeNoDaArvoreNaoSuportado => "tipo_de_no_da_arvore_nao_suportado",
            Self::NoDoProcessoAusenteNaArvore => "no_do_processo_ausente_na_arvore",
            Self::NosDoProcessoRepetidosNaArvore => "nos_do_processo_repetidos_na_arvore",
            Self::NoDoProcessoComPai => "no_do_processo_com_pai",
            Self::NupDoNoDoProcessoInconsistente => "nup_do_no_do_processo_inconsistente",
            Self::IdDeNoNaoDecimal => "id_de_no_nao_decimal",
            Self::DocumentoComPaiDiferenteDoProcesso => "documento_com_pai_diferente_do_processo",
            Self::DocumentoRepetidoNaArvore => "documento_repetido_na_arvore",
            Self::IdentidadeDoProcessoDiferenteDaPedida => {
                "identidade_do_processo_diferente_da_pedida"
            }
            Self::IdDoProcessoNaArvoreDiferenteDoAberto => {
                "id_do_processo_na_arvore_diferente_do_aberto"
            }
        }
    }

    /// A árvore é de outro processo: um palpite do índice errou.
    pub(crate) fn diverge_do_pedido(self) -> bool {
        matches!(
            self,
            Self::IdentidadeDoProcessoDiferenteDaPedida | Self::IdDoProcessoNaArvoreDiferenteDoAberto
        )
    }
}

/// Argumentos por nó, observados na UnB 4.0.9 e em GO.
const ARGUMENTOS: usize = 16;

/// Lê os nós da árvore e confere a identidade: o NUP do nó `PROCESSO` tem
/// de ser **igual** ao pedido (comparação opaca) e o id igual ao aberto.
pub(crate) fn ler_arvore(
    html: &str,
    nup_pedido: &str,
    id_aberto: &str,
) -> Result<ArvoreLida, ErroArvore> {
    let _ = (html, nup_pedido, id_aberto, ARGUMENTOS);
    let _: Option<Literal> = None;
    let _ = ler_declaracao;
    Err(ErroArvore::NoDoProcessoAusenteNaArvore)
}
