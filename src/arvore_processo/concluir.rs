//! A ação "Concluir Processo" publicada na barra do nó `PROCESSO` e, na
//! forma do SEI 4.0, a URL do corpo de `concluirProcesso()` na tela do nó
//! (contracts/cli/processo-concluir.md; observação em
//! docs/como-o-sei-funciona/conclusao-do-processo.md). Duas formas:
//! - 4.1 e 5.0 (UFRJ, PEN2, GO, PEN3, PEN4): `<a href="controlador.php?
//!   acao=procedimento_concluir&…">`, link da tela "Conclusão de Processo";
//! - 4.0 (`UnB` 4.0.9, PEN1 4.0.12): `<a href="#"
//!   onclick="concluirProcesso();">`; a URL está na tela do nó
//!   (`arvore_visualizar`, o `href` a3 do `Nos[i]` `PROCESSO`), em três
//!   linhas: a da função, a do `confirm` comentado e a do `location.href`.
//!
//! Sem regex e sem JavaScript executado: só essas formas exatas.
use super::barra_do_processo::{FalhaDaBarra, acao_do_href, ancoras_da_barra};
use super::endereco::{EnderecoRecusado, conferir_endereco_relativo};
use super::literais::{Literal, ler_declaracao};
use crate::instalacao::Instalacao;

/// A `acao` da conclusão nas duas formas.
pub(crate) const ACAO_DE_CONCLUIR: &str = "procedimento_concluir";
/// A `acao` da tela do nó, onde a forma 4.0 publica a URL.
pub(crate) const ACAO_DA_TELA_DO_NO: &str = "arvore_visualizar";
const ONCLICK: &str = "concluirProcesso();";

/// URL absoluta e assinada: `Debug` redigido, sem `Display`.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct EnderecoDeEscrita(String);

impl EnderecoDeEscrita {
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for EnderecoDeEscrita {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EnderecoDeEscrita(<redigido>)")
    }
}

/// Como a barra oferece concluir.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ConcluirNaBarra {
    /// Link da tela "Conclusão de Processo" (forma `tela`).
    PelaTela(EnderecoDeEscrita),
    /// `concluirProcesso()` (forma `funcao`): a tela do nó, onde está a URL.
    PelaFuncao { tela_do_no: EnderecoDeEscrita },
}

/// Recusas da leitura da barra (só pesam quando se pede para concluir).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroConcluirNaBarra {
    BarraDoProcessoForaDaFormaObservada,
    BarrasDoProcessoRepetidasNaArvore,
    /// Dois links, duas funções, ou link e função juntos.
    FormasDeConcluirRepetidasNaBarra,
    EnderecoDaConclusaoNaoReconhecido,
    EnderecoDaConclusaoDeOutroProcesso,
    /// Forma `funcao` sem o `href` assinado da tela do nó na raiz.
    EnderecoDaTelaDoNoNaoReconhecido,
}

impl ErroConcluirNaBarra {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::BarraDoProcessoForaDaFormaObservada => {
                "barra_do_processo_fora_da_forma_observada"
            }
            Self::BarrasDoProcessoRepetidasNaArvore => "barras_do_processo_repetidas_na_arvore",
            Self::FormasDeConcluirRepetidasNaBarra => "formas_de_concluir_repetidas_na_barra",
            Self::EnderecoDaConclusaoNaoReconhecido => "endereco_da_conclusao_nao_reconhecido",
            Self::EnderecoDaConclusaoDeOutroProcesso => "endereco_da_conclusao_de_outro_processo",
            Self::EnderecoDaTelaDoNoNaoReconhecido => "endereco_da_tela_do_no_nao_reconhecido",
        }
    }
}

/// A ação de concluir da barra do processo `id`; `Ok(None)` se a barra
/// não oferece concluir (nem link nem função).
pub(crate) fn ler_concluir_da_barra(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<Option<ConcluirNaBarra>, ErroConcluirNaBarra> {
    let _ = (
        instalacao,
        html,
        id,
        ONCLICK,
        ancoras_da_barra,
        acao_do_href,
    );
    let _: Option<FalhaDaBarra> = None;
    let _ = (conferir_endereco_relativo, ler_declaracao);
    let _: Option<(EnderecoRecusado, Literal)> = None;
    Ok(None)
}

/// Recusas da leitura de `concluirProcesso()` na tela do nó.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ErroFuncaoDeConcluir {
    FuncaoDeConcluirAusenteNaTelaDoNo,
    FuncaoDeConcluirForaDaFormaObservada,
    FuncoesDeConcluirRepetidasNaTelaDoNo,
    EnderecoDaConclusaoNaoReconhecido,
    EnderecoDaConclusaoDeOutroProcesso,
}

impl ErroFuncaoDeConcluir {
    pub(crate) fn subcodigo(self) -> &'static str {
        match self {
            Self::FuncaoDeConcluirAusenteNaTelaDoNo => "funcao_de_concluir_ausente_na_tela_do_no",
            Self::FuncaoDeConcluirForaDaFormaObservada => {
                "funcao_de_concluir_fora_da_forma_observada"
            }
            Self::FuncoesDeConcluirRepetidasNaTelaDoNo => {
                "funcoes_de_concluir_repetidas_na_tela_do_no"
            }
            Self::EnderecoDaConclusaoNaoReconhecido => "endereco_da_conclusao_nao_reconhecido",
            Self::EnderecoDaConclusaoDeOutroProcesso => "endereco_da_conclusao_de_outro_processo",
        }
    }
}

/// A URL de `concluirProcesso()` na tela do nó do processo `id`, conferida.
pub(crate) fn ler_funcao_de_concluir(
    instalacao: &Instalacao,
    html: &str,
    id: &str,
) -> Result<EnderecoDeEscrita, ErroFuncaoDeConcluir> {
    let _ = (instalacao, html, id);
    Err(ErroFuncaoDeConcluir::FuncaoDeConcluirAusenteNaTelaDoNo)
}
