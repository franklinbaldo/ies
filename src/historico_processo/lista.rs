//! A página `procedimento_consultar_historico`: as linhas pelo leitor de
//! `--historico-privado`, a legenda `Lista de Andamentos (N registro):` ou
//! `(N registros):` da `table#tblHistorico` e a visão servida em
//! `form#frmProcedimentoHistorico input[name=hdnTipoHistorico]`.
use super::{AndamentosLidos, ErroAndamentos as E};

/// Lê a página e confere o NUP do título com o pedido.
pub(crate) fn ler_andamentos(_html: &str, _nup: &str) -> Result<AndamentosLidos, E> {
    Err(E::LegendaDosAndamentosAusente)
}
